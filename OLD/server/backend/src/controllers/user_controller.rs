
use actix_web::{ web, HttpRequest, HttpResponse, };
use common::*;
use crate::AKData;
use crate::data_processor::OutUserData;
use crate::db_utils;
use crate::models::user;
use futures::{ future::err, future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use actix_identity::Identity;
use crate::ak_error::AkError;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
    include_contacts: Option<bool>,
}

pub fn users_status(_uid: Identity, state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    s.data_processor
        .send(OutUserData)
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

pub fn get_user(uid: Identity, state: AKData, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    let fut = if prefetch {
                        Either::A(user::select_user_prefetch(client, id))
                    } else {
                        Either::B(user::select_user(client, id))
                    };

                    ok::<_, AkError>(fut).flatten()
                })
                .and_then(|(_client, opt_user, opt_e_user)| {
                    match opt_user {
                        Some(u) => {
                            ok(HttpResponse::Ok().json(Ok::<_, AkError>((u, opt_e_user))))
                        },
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

pub fn get_users(uid: Identity, state: AKData, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let include_contacts = params.include_contacts.unwrap_or(true);
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            user::select_users(client, include_contacts)
        })
        .map(|(_client, users)| {
            HttpResponse::Ok().json(Ok::<_, AkError>(users))
        })
}

// new user
pub fn post_user(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let (user, opt_e_user) = payload.into_inner();

    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            user::insert_user(client, user)
                .and_then(move |(client, opt_user)| {
                    match &opt_user {
                        Some(created_user) => {
                            match opt_e_user {
                                Some(mut e_user) => {
                                    e_user.attached_user = Some(created_user.id);
                                    Either::A(user::insert_user(client, e_user)
                                        .map(move |(client, created_e_user)| {
                                            (client, opt_user, created_e_user)
                                        })
                                    )
                                },
                                None => Either::B(ok((client, opt_user, None))),
                            }

                        },
                        None => {
                            Either::B(ok((client, None, None)))
                        },
                    }
                })
        })
        .and_then(|(_client, user, opt_e_user)| {
            match user {
                Some(u) => ok(HttpResponse::Ok().json(Ok::<_, AkError>((u, opt_e_user)))),
                None => err(AkError::not_found()),
            }
        })
}

pub fn put_user(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let (user, opt_e_user) = payload.into_inner();

    db_utils::connect_id(&uid, &state)
        .and_then(|client| {
            user::update_user(client, user)
                .and_then(move |(client, opt_user)| {
                    match opt_e_user {
                        Some(e_user) => {
                            let fut = if e_user.id != -1 {
                                Either::A(user::update_user(client, e_user))
                            } else {
                                Either::B(user::insert_user(client, e_user))
                            };

                            Either::A(fut.map(move |(client, opt_e_user)| {
                                    (client, opt_user, opt_e_user)
                                })
                            )
                        },
                        None => {
                            Either::B(ok((client, opt_user, None)))
                        }
                    }
                })
        })
        .and_then(|(_client, opt_user, opt_e_user)| {
            match opt_user {
                Some(u) => ok(HttpResponse::Ok().json(Ok::<_, AkError>((u, opt_e_user)))),
                None => err(AkError::not_found()),
            }
        })
}

pub fn delete_user(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    user::delete_user(client, id)
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

