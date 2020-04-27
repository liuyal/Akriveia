
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use actix_web::web::{ BytesMut, };
use crate::ak_error::AkError;

pub fn row_to_map(row: &Row) -> Map {
    let mut entry = Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "m_id" => entry.id = row.get(i),
            "m_blueprint" => {}, // handle BYTEA differently
            "m_bounds" => {
                let bounds: Vec<i32> = row.get(i);
                entry.bounds = na::Vector2::new(bounds[0], bounds[1]);
            }
            "m_scale" => entry.scale = row.get(i),
            "m_name" => entry.name = row.get(i),
            "m_note" => entry.note = row.get(i),
            unhandled if unhandled.starts_with("m_") => { panic!("unhandled beacon column {}", unhandled); },
            _ => {},
        }
    }
    entry
}

pub fn maybe_row_to_map(row: &Row) -> Option<Map> {
    let mut found = false;
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "m_id" => {
                found = true;
                let id: Option<i32> = row.get(i);
                if id.is_none() {
                    return None;
                }
            }
            _ => {}
        }
    }

    if found {
        Some(row_to_map(row))
    } else {
        None
    }
}

pub fn select_maps(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<Map>), Error=AkError> {
    // TODO paging
    client
        .prepare("
            SELECT m_id, m_bounds, m_scale, m_name, m_note FROM runtime.maps
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_map(&row)).collect())
                })
        })
}

pub fn select_map(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=AkError> {
    client
        .prepare("
            SELECT m_id, m_bounds, m_scale, m_name, m_note FROM runtime.maps
            WHERE m_id = $1::INTEGER
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_map_blueprint(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<Vec<u8>>), Error=AkError> {
    client
        .prepare_typed("
            SELECT m_id, m_blueprint FROM runtime.maps
            WHERE m_id = $1
        ", &[
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => {
                            for (i, column) in r.columns().iter().enumerate() {
                                match column.name() {
                                    "m_blueprint" => {
                                        return (client, r.get(i))
                                    },
                                    _ => {},
                                }
                            }
                            (client, None)
                        },
                        None => (client, None),
                    }
                })
        })
}

pub fn insert_map(mut client: tokio_postgres::Client, map: Map) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=AkError> {
    client
        .prepare_typed("
            INSERT INTO runtime.maps (
                m_bounds,
                m_name,
                m_note,
                m_scale
            )
            VALUES( $1, $2, $3, $4 )
            RETURNING m_id, m_bounds, m_scale, m_name, m_note
        ", &[
            Type::INT4_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::FLOAT8,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let bounds = vec![map.bounds[0], map.bounds[1]];
            client
                .query(&statement, &[
                    &bounds,
                    &map.name,
                    &map.note,
                    &map.scale,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_map(mut client: tokio_postgres::Client, map: Map) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.maps
            SET
                m_bounds = $1,
                m_name = $2,
                m_note = $3,
                m_scale = $4
             WHERE
                m_id = $5
            RETURNING m_id, m_bounds, m_scale, m_name, m_note

        ", &[
            Type::INT4_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::FLOAT8,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let bounds = vec![map.bounds[0], map.bounds[1]];
            client
                .query(&statement, &[
                    &bounds,
                    &map.name,
                    &map.note,
                    &map.scale,
                    &map.id,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_map_blueprint(mut client: tokio_postgres::Client, mid: i32, img: BytesMut) -> impl Future<Item=(tokio_postgres::Client), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.maps
            SET
                m_blueprint = $1
             WHERE
                m_id = $2
        ", &[
            Type::BYTEA,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[
                    &img.as_ref(),
                    &mid,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(_row, _next)| {
                    client
                })
        })
}

pub fn delete_map(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=AkError> {
    client
        .prepare_typed("
            DELETE FROM runtime.maps
            WHERE (
                m_id = $1
            )
        ", &[
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(_row, _next)| {
                    client
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_utils;
    use tokio::runtime::current_thread::Runtime;

    #[test]
    fn insert() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let task = db_utils::default_connect()
            .map_err(AkError::from)
            .and_then(|client| {
                insert_map(client, map)
            })
            .map(|(_client, _opt_map)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert map");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update_blueprint() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();

        let task = db_utils::default_connect()
            .map_err(AkError::from)
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                let id = opt_map.unwrap().id;
                let file: BytesMut = (1..255).collect();
                update_map_blueprint(client, id, file)
            })
            .map(|_client| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();
        let mut updated_map = map.clone();
        updated_map.name = "map_1".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                updated_map.id = opt_map.unwrap().id;
                update_map(client, updated_map)
            })
            .map(|(_client, _beacon)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                select_map(client, opt_map.unwrap().id)
            })
            .map(|(_client, _opt_map)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_blueprint() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();

        let file: BytesMut = (1..255).collect();
        let file2 = file.clone();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                let id = opt_map.unwrap().id;
                update_map_blueprint(client, id, file)
                    .map(move |client| {
                        (client, id)
                    })
            })
            .and_then(|(client, id)| {
                select_map_blueprint(client, id)
            })
            .map(move |(_client, data)| {
                data.unwrap().iter().zip(file2).for_each(|(&a, b)| {
                    assert!(a == b);
                });
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_many() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, _opt_map)| {
                select_maps(client)
            })
            .map(|(_client, _maps)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn delete() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut map = Map::new();
        map.name = "map_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                delete_map(client, opt_map.unwrap().id)
            })
            .map(|_client| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }
}
