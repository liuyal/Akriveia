
use actix_web::{ web, HttpRequest, HttpResponse, };
use common::*;
use crate::AKData;
use crate::db_utils;
use crate::models::map;
use futures::{ Stream, future::err, future::ok, Future, future::Either, };
use actix_identity::Identity;
use crate::ak_error::AkError;

pub fn get_map(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::select_map(client, id)
                })
                .and_then(|(_client, map)| {
                    match map {
                        Some(m) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(m))),
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

pub fn get_map_blueprint(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::select_map_blueprint(client, id)
                })
                .map(|(_client, res)| {
                    match res {
                        Some(img) => HttpResponse::Ok().body(img),
                        None => HttpResponse::NotFound().finish(),
                    }
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_maps(uid: Identity, state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::select_maps(client)
        })
        .map(|(_client, maps)| {
            HttpResponse::Ok().json(Ok::<_, AkError>(maps))
        })
}

// new map
pub fn post_map(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::insert_map(client, payload.0)
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(m))),
                None => err(AkError::not_found()),
            }
        })
}

// update map
pub fn put_map(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::update_map(client, payload.0)
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => ok(HttpResponse::Ok().json(Ok::<_, AkError>(m))),
                None => err(AkError::not_found()),
            }
        })
}

// update map blueprint
pub fn put_map_blueprint(uid: Identity, state: AKData, req: HttpRequest, payload: web::Payload) -> impl Future<Item=HttpResponse, Error=AkError> {
    let mid = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match mid {
        Ok(id) => {
            Either::A(payload
                .map_err(AkError::from)
                .fold(web::BytesMut::new(), move |mut acc_body, chunk| {
                    acc_body.extend_from_slice(&chunk);
                    Ok::<_, AkError>(acc_body)
                })
                .and_then(move |blueprint_img| {
                    db_utils::connect_id(&uid, &state)
                        .and_then(move |client| {
                            map::update_map_blueprint(client, id, blueprint_img)
                        })
                })
                .map(|_client| {
                    HttpResponse::Ok().finish()
                })
            )
        },
        _ => {
            Either::B(err(AkError::not_found()))
        }
    }
}

pub fn delete_map(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::delete_map(client, id)
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

