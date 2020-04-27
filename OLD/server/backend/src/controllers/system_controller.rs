use actix::System;
use actix_identity::Identity;
use actix_web::{ web, HttpRequest, HttpResponse, };
use common::*;
use crate::AKData;
use crate::WatcherCommand;
use crate::beacon_manager::{ BMCommand, GetDiagnosticData, };
use futures::{ future::err, future::ok, Future, };
use actix::Arbiter;
use std::time::Duration;
use crate::ak_error::AkError;

pub fn post_emergency(state: AKData, _req: HttpRequest, payload: web::Json<SystemCommandResponse>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    let command = if payload.emergency {
        BMCommand::StartEmergency(None)
    } else {
        BMCommand::EndEmergency(None)
    };

    s.beacon_manager
        .send(command)
        .then(|res| {
            match res {
                Ok(data) => {
                    ok(HttpResponse::Ok().json(data))
                },
                _ => {
                    err(AkError::internal())
                }
        }})
}

pub fn get_emergency(state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(BMCommand::GetEmergency)
        .then(|res| {
            match res {
                Ok(data) => {
                    ok(HttpResponse::Ok().json(data))
                },
                _ => {
                    err(AkError::internal())
                }
        }})
}

pub fn diagnostics(state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(GetDiagnosticData)
        .then(|res| {
            match res {
                Ok(data) => {
                    ok(HttpResponse::Ok().json(data))
                },
                _ => {
                    err(AkError::internal())
                }
        }})
}

pub fn ping(_req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    ok(HttpResponse::Ok().json(Ok::<_, AkError>(())))
}

pub fn restart(id: Identity, state: AKData, payload: web::Json<SystemCommand>) -> Result<HttpResponse, AkError> {
    if let Some(name) = id.identity() {
        if name == "admin" {
            let s = state.lock().unwrap();
            let command = match payload.0 {
                SystemCommand::StartNormal => WatcherCommand::StartNormal,
                SystemCommand::RebuildDB => WatcherCommand::RebuildDB,
                SystemCommand::RebuildDemoDB => WatcherCommand::RebuildDemoDB,
            };
            match s.tx.send(command) {
                Ok(()) => {},
                Err(e) => {
                    println!("Failed to notify watcher we are shutting down {}", e);
                },
            }

            // HACK, attempt to give the request enough time to reply to the client before
            // shutting down
            println!("initiating shutdown");
            let shutdown_fut = tokio::timer::Delay::new(tokio::clock::now() + Duration::from_millis(500))
                .map(|_| {
                    println!("shutting down now");
                    let system = System::current();
                    system.stop();
                })
                .map_err(|_e| {
                });
            Arbiter::spawn(shutdown_fut);

            Ok(HttpResponse::Ok().finish())
        } else {
            Err(AkError::unauthorized())
        }
    } else {
        Err(AkError::unauthorized())
    }
}
