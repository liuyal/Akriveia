use actix_web::{ web, HttpRequest, HttpResponse, };
use crate::AKData;
use crate::beacon_manager::{ OutBeaconData, BMCommand, };
use crate::db_utils;
use crate::models::beacon;
use futures::{ future::err, future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use actix_identity::Identity;
use common::BeaconRequest;
use crate::ak_error::AkError;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
}

pub fn beacons_status(_uid: Identity, state: AKData) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(OutBeaconData)
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

pub fn beacon_command(_uid: Identity, state: AKData, payload: web::Json<common::BeaconRequest>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    let command = match payload.0 {
        BeaconRequest::StartEmergency(mac) => BMCommand::StartEmergency(mac),
        BeaconRequest::EndEmergency(mac) => BMCommand::EndEmergency(mac),
        BeaconRequest::Ping(mac) => BMCommand::Ping(mac),
        BeaconRequest::Reboot(mac) => BMCommand::Reboot(mac),
        BeaconRequest::SetIp(ip) => BMCommand::SetIp(ip),
    };
    s.beacon_manager
        .send(command)
        .then(|res| {
            match res {
                Ok(_data) => {
                    ok(HttpResponse::Ok().json(Ok::<_, AkError>(())))
                },
                _ => {
                    err(AkError::internal())
                }
        }})
}

pub fn get_beacon(uid: Identity, state: AKData, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    let fut = if prefetch {
                        Either::A(beacon::select_beacon_prefetch(client, id))
                    } else {
                        Either::B(beacon::select_beacon(client, id))
                    };

                    ok::<_, AkError>(fut).flatten()
                })
                .and_then(|(_client, beacon_and_map)| {
                    match beacon_and_map {
                        Some((beacon, opt_map)) => {
                            match opt_map {
                                Some(map) => ok(HttpResponse::Ok().json(Ok::<_, AkError>((beacon, Some(map))))),
                                None => ok(HttpResponse::Ok().json(Ok::<_, AkError>(beacon))),
                            }

                        },
                        None => err(AkError::not_found()),
                    }
                })
            )
        },
        _ => {
            Either::B(err(AkError::not_found()))
        }
    }
}

pub fn get_beacons_for_map(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").and_then(|value| value.parse::<i32>().ok());
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::select_beacons_for_map(client, id)
        })
        .map(|(_client, beacons)| {
            HttpResponse::Ok().json(Ok::<_, AkError>(beacons))
        })
}

pub fn get_beacons(uid: Identity, state: AKData, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let prefetch = params.prefetch.unwrap_or(false);
    let connect = db_utils::connect_id(&uid, &state);

    if prefetch {
        Either::A(
            connect
                .and_then(move |client| {
                    beacon::select_beacons_prefetch(client)
                })
                .map(|(_client, beacons_and_maps)| {
                    HttpResponse::Ok().json(Ok::<_, AkError>(beacons_and_maps))
                })
        )
    } else {
        Either::B(
            connect
                .and_then(move |client| {
                    beacon::select_beacons(client)
                })
                .map(|(_client, beacons)| {
                    HttpResponse::Ok().json(Ok::<_, AkError>(beacons))
                })
        )
    }
}

// new beacon
pub fn post_beacon(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::insert_beacon(client, payload.0)
        })
        .and_then(|(_client, beacon)| {
            match beacon {
                Some(b) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(b))),
                None => err(AkError::not_found()),
            }
        })
}

// update beacon
pub fn put_beacon(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::update_beacon(client, payload.0)
        })
        .and_then(|(_client, beacon)| {
            match beacon {
                Some(b) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(b))),
                None => err(AkError::not_found()),
            }
        })
}

pub fn delete_beacon(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    beacon::delete_beacon(client, id)
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

