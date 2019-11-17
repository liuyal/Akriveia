use tokio_postgres::NoTls;
use common::LoginInfo;
use futures::{ future::err, Future, future::Either, };
use crate::AKData;
use actix_web::{ error, web, };
use std::sync::Mutex;
use actix_identity::Identity;

pub const DEFAULT_CONNECTION: &str = "dbname=ak host=localhost password=postgres user=postgres";

pub fn default_connect() -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(DEFAULT_CONNECTION)
}

pub fn connect_login(user: &LoginInfo) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(&format!("dbname=ak host=localhost user={} password={}", user.name, user.pw))
}

pub fn connect(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}

pub fn connect_id(id: &Identity, state: &AKData) -> impl Future<Item=tokio_postgres::Client, Error=actix_web::Error> {
    let conn_fut = if let Some(name) = id.identity() {
        let s = state.lock().unwrap();
        if let Some(info) = s.pools.get(&name) {
            Either::A(connect_login(info)
                .map_err(|postgres_err| {
                    error::ErrorBadRequest(postgres_err)
                })
            )
        } else {
            Either::B(err(error::ErrorInternalServerError("Empty connection pool for valid user.")))
        }
    } else {
        Either::B(err(error::ErrorUnauthorized("invalid credentials")))
    };

    conn_fut
}
