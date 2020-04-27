#![deny(warnings)]
extern crate actix;
extern crate actix_files;
extern crate actix_identity;
extern crate actix_session;
extern crate actix_web;
extern crate chrono;
extern crate common;
extern crate env_logger;
extern crate eui48;
extern crate eui64;
extern crate futures;
extern crate ipc_channel;
extern crate ipnet;
extern crate libc;
extern crate nalgebra as na;
extern crate tokio_postgres;

mod beacon_manager;
mod beacon_udp;
mod dummy_udp;
mod controllers;
mod data_processor;
mod db_utils;
mod models;
mod conn_common;
mod ak_error;

use controllers::beacon_controller;
use controllers::map_controller;
use controllers::network_interface_controller;
use controllers::session_controller;
use controllers::system_controller;
use controllers::user_controller;

use models::system;

use actix::prelude::*;
use actix_files as fs;
use actix_identity::{ CookieIdentityPolicy, IdentityService, };
use actix_web::{ error, middleware, web, App, HttpRequest, HttpResponse, HttpServer, };
use beacon_manager::*;
use common::*;
use data_processor::*;
use ipc_channel::ipc::{ self, IpcReceiver, IpcSender, };
use serde_derive::{ Deserialize, Serialize, };
use std::collections::HashMap;
use std::env;
use std::sync::*;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum WatcherCommand {
    StartNormal,
    RebuildDB,
    RebuildDemoDB,
}

pub struct AkriveiaState {
    pub tx: ipc::IpcSender<WatcherCommand>,
    pub rx: ipc::IpcReceiver<SystemCommand>,
    pub beacon_manager: Addr<BeaconManager>,
    pub data_processor: Addr<DataProcessor>,
    // I would prefer this was a per user connection pool,
    // but r2d2 does not work for tokio, and bb8 does not look very mature.
    pub pools: HashMap<String, LoginInfo>,
}

pub type AKData = web::Data<Arc<Mutex<AkriveiaState>>>;

impl AkriveiaState {
    pub fn new(tx: IpcSender<WatcherCommand>, rx: IpcReceiver<SystemCommand>) -> AKData {
        let data_processor_addr =  DataProcessor::new().start();
        let beacon_manager_addr = BeaconManager::new(data_processor_addr.clone());

        beacon_manager_addr.do_send(BMCommand::ScanBeacons);

        web::Data::new(Arc::new(Mutex::new(AkriveiaState {
            beacon_manager: beacon_manager_addr,
            data_processor: data_processor_addr,
            pools: HashMap::new(),
            tx,
            rx,
        })))
    }
}

fn default_route(req: HttpRequest) -> HttpResponse {
    println!("default route called");
    println!("request was: {:?}", req);
    HttpResponse::NotFound().finish()
}

fn webserver_main(start_command: SystemCommand, tx: IpcSender<WatcherCommand>, rx: IpcReceiver<SystemCommand>) {
    let system = System::new("Akriviea");
    env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    match start_command {
        SystemCommand::StartNormal => {},
        SystemCommand::RebuildDB => {
            let create_db_fut = system::create_db(false);
            // intentionally block all further execution
            tokio::run(create_db_fut);
        },
        SystemCommand::RebuildDemoDB => {
            let create_db_fut = system::create_db(true);
            // intentionally block all further execution
            tokio::run(create_db_fut);
        },
    }

    let state = AkriveiaState::new(tx, rx);

    // start the webserver
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("session_token")
                    .secure(false)
            ))
            .data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, req| {
                        println!("Failed to parse request body {:?},\n {:?}", err, req);
                        error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().finish()
                        ).into()
                    })
            )
            .register_data(state.clone())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            // beacon
            .service(
                web::resource(&beacons_url())
                    .route(web::get().to_async(beacon_controller::get_beacons))
            )

            .service(
                web::resource(&beacon_url("{id}"))
                    .route(web::get().to_async(beacon_controller::get_beacon))
                    .route(web::put().to_async(beacon_controller::put_beacon))
                    .route(web::delete().to_async(beacon_controller::delete_beacon))
            )
            .service(
                web::resource(&beacon_url(""))
                    .route(web::post().to_async(beacon_controller::post_beacon))
            )
            .service(
                web::resource(&beacons_status_url())
                    .to_async(beacon_controller::beacons_status)
            )
            .service(
                web::resource(&beacon_command_url())
                    .to_async(beacon_controller::beacon_command)
            )


            // user
            .service(
                web::resource(&users_url())
                    .route(web::get().to_async(user_controller::get_users))
            )
            .service(
                web::resource(&user_url("{id}"))
                    .route(web::get().to_async(user_controller::get_user))
                    .route(web::put().to_async(user_controller::put_user))
                    .route(web::delete().to_async(user_controller::delete_user))
            )
            .service(
                web::resource(&user_url(""))
                    .route(web::post().to_async(user_controller::post_user))
            )
            .service(
                web::resource(&users_status_url())
                    .to_async(user_controller::users_status)
            )

            // map
            .service(
                web::resource(&maps_url())
                    .route(web::get().to_async(map_controller::get_maps))
            )
            .service(
                web::resource(&beacons_for_map_url("{id}"))
                    .route(web::get().to_async(beacon_controller::get_beacons_for_map))
            )
            .service(
                web::resource(&map_url("{id}"))
                    .route(web::get().to_async(map_controller::get_map))
                    .route(web::put().to_async(map_controller::put_map))
                    .route(web::delete().to_async(map_controller::delete_map))
            )
            .service(
                web::resource(&map_blueprint_url("{id}"))
                    .route(web::put().to_async(map_controller::put_map_blueprint))
                    .route(web::get().to_async(map_controller::get_map_blueprint))
            )
            .service(
                web::resource(&map_url(""))
                    .route(web::post().to_async(map_controller::post_map))
            )

            // system
            .service(
                web::resource(&system_emergency_url())
                    .route(web::get().to_async(system_controller::get_emergency))
                    .route(web::post().to_async(system_controller::post_emergency))
            )
            .service(
                web::resource(&system_diagnostics_url())
                    .route(web::get().to_async(system_controller::diagnostics))
            )
            .service(
                web::resource(&system_restart_url())
                    .route(web::post().to(system_controller::restart))
            )
            .service(
                web::resource(&system_ping_url())
                    .route(web::get().to_async(system_controller::ping))
            )

            // network
            .service(
                web::resource(&networks_url())
                    .route(web::get().to_async(network_interface_controller::get_network_interfaces))
            )
            .service(
                web::resource(&network_url("{id}"))
                    .route(web::get().to_async(network_interface_controller::get_network_interface))
                    .route(web::put().to_async(network_interface_controller::put_network_interface))
                    .route(web::delete().to_async(network_interface_controller::delete_network_interface))
            )
            .service(
                web::resource(&network_url(""))
                    .route(web::post().to_async(network_interface_controller::post_network_interface))
            )

            // session
            .service(
                web::resource(&session_check_url())
                    .route(web::get().to_async(session_controller::check))
            )
            .service(
                web::resource(&session_login_url())
                    .route(web::post().to_async(session_controller::login))
            )
            .service(
                web::resource(&session_logout_url())
                    .route(web::post().to_async(session_controller::logout))
            )
            // these two last !!
            .service(fs::Files::new("/", "static").index_file("index.html"))
            .default_service(web::resource("").to(default_route))
    })
    .bind("0.0.0.0:8080").unwrap()
    .start();

    let sys_result = system.run();
    match sys_result {
        Err(e) => {
            println!("actix system error: {}", e);
        },
        _ => {},
    }
}

fn watch(_tx: IpcSender<SystemCommand>, rx: IpcReceiver<WatcherCommand>) -> SystemCommand {
    // just do nothing and wait on a response from the webserver.
    // maybe later implement pinging.
    loop {
        match rx.recv() {
            Ok(command) => {
                match command {
                    WatcherCommand::StartNormal => {
                        return SystemCommand::StartNormal;
                    },
                    WatcherCommand::RebuildDB => {
                        return SystemCommand::RebuildDB;
                    },
                    WatcherCommand::RebuildDemoDB => {
                        return SystemCommand::RebuildDemoDB;
                    },
                }
            },
            Err(e) => {
                println!("watcher error with recv communication, {}", e);
                // TODO restart server at this point?
            },
        }
    }
}

fn main() {
    let mut start_command = SystemCommand::StartNormal;

    loop {
        let (child_tx, parent_rx) = ipc::channel::<WatcherCommand>().unwrap();
        let (parent_tx, child_rx) = ipc::channel::<SystemCommand>().unwrap();

        let pid = unsafe { libc::fork() };
        match pid {
            0 => {
                webserver_main(start_command, child_tx, child_rx);
                return;
            },
            -1 => {
                panic!("failed to create child");
            },
            child_pid => {
                start_command = watch(parent_tx, parent_rx);
                unsafe {
                    let mut status = 0;
                    let pid = libc::waitpid(child_pid, &mut status, 0);
                    assert!(pid == child_pid);
                }
            },
        };
    }
}

