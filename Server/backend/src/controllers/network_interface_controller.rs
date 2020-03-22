use actix_web::{ web, HttpRequest, HttpResponse, };
use crate::AKData;
use common::NetworkInterface;
use crate::db_utils;
use crate::models::network_interface;
use futures::{ future::err, future::ok, Future, future::Either, };
use crate::ak_error::AkError;

pub fn get_network_interface(_state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    network_interface::select_network_interface(client, id)
                })
                .and_then(|(_client, iface)| {
                    match iface {
                        Some(u) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(u))),
                        None => err(AkError::not_found()),
                    }
                })
            )
        },
        _ => {
            Either::B(err(AkError::not_found()))
        },
    }
}

pub fn get_network_interfaces(_state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::select_network_interfaces(client)
        })
        .map(|(_client, ifaces)| {
            HttpResponse::Ok().json(Ok::<_, AkError>(ifaces))
        })
}

// new iface
pub fn post_network_interface(_state: AKData, _req: HttpRequest, payload: web::Json<NetworkInterface>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::insert_network_interface(client, payload.0)
        })
        .and_then(|(_client, iface)| {
            match iface {
                Some(b) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(b))),
                None => err(AkError::not_found())
            }
        })
}

// update iface
pub fn put_network_interface(_state: AKData, _req: HttpRequest, payload: web::Json<NetworkInterface>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::update_network_interface(client, payload.0)
        })
        .and_then(|(_client, iface)| {
            match iface {
                Some(b) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(b))),
                None => err(AkError::not_found()),
            }
        })
}

pub fn delete_network_interface(_state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    network_interface::delete_network_interface(client, id)
                })
                .map(|_client| {
                    HttpResponse::Ok().json(Ok::<_, AkError>(()))
                })
            )
        },
        _ => {
            Either::B(err(AkError::not_found()))
        }
    }
}

