
use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use actix_identity::Identity;
use common::*;
use crate::AKData;
use crate::db_utils;
use futures::future::{ err, Either, ok, Future, };
use crate::ak_error::AkError;

pub fn login(id: Identity, state: AKData, payload: web::Json<LoginInfo>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    if payload.name == "responder" {
        let mut info = LoginInfo::new();
        info.name = payload.name.clone();
        info.pw = payload.name.clone();
        id.remember(payload.name.clone());
        let mut s = state.lock().unwrap();
        s.pools.insert(info.name.clone(), info);
        Either::A(ok(HttpResponse::Ok().finish()))
    } else {
        Either::B(db_utils::connect_login(&payload.0)
            .map(move |_client| {
                id.remember(payload.name.clone());
                let mut s = state.lock().unwrap();
                s.pools.insert(payload.name.clone(), payload.0);
                HttpResponse::Ok().finish()
            })
            .map_err(|_postgres_err| {
                AkError::unauthorized()
            })
        )
    }
}

pub fn check(id: Identity, _state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    if let Some(_name) = id.identity() {
        ok(HttpResponse::Ok().finish())
    } else {
        err(AkError::unauthorized())
    }
}

pub fn logout(id: Identity, _state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    id.forget();
    ok(HttpResponse::Ok().finish())
}

