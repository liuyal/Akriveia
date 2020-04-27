
use common::*;
use crate::models::map;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use std::net::IpAddr;
use crate::ak_error::AkError;

pub fn row_to_beacon(row: &Row) -> Beacon {
    let mut b = Beacon::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "b_id" => b.id = row.get(i),
            "b_mac_address" => b.mac_address = row.get(i),
            "b_ip" => b.ip = row.get(i),
            "b_coordinates" => {
                let coordinates: Vec<f64> = row.get(i);
                b.coordinates = na::Vector2::new(coordinates[0], coordinates[1]);
            }
            "b_last_active" => b.last_active = row.get(i),
            "b_map_id" => b.map_id = row.get(i),
            "b_name" => b.name = row.get(i),
            "b_note" => b.note = row.get(i),
            "b_state" => b.state = BeaconState::from(row.get::<usize, i16>(i)),
            unhandled if unhandled.starts_with("b_") => { panic!("unhandled beacon column {}", unhandled); },
            _ => {},
        }
    }
    b
}

pub fn select_beacons(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<Beacon>), Error=AkError> {
    // TODO paging
    client
        .prepare("
            SELECT * FROM runtime.beacons
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_beacon(&row)).collect())
                })
        })
}

pub fn select_beacons_prefetch(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<(Beacon, Option<Map>)>), Error=AkError> {
    // TODO paging
    client
        .prepare("
            SELECT *
            FROM runtime.beacons AS beacon
            LEFT JOIN runtime.maps AS map ON map.m_id = beacon.b_map_id
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (
                        client,
                        rows
                            .into_iter()
                            // this works because the row conversion functions only
                            // look for entries specific to the object they are
                            // converting for, and the keys are all unique.
                            .map(|row| {
                                 (row_to_beacon(&row), map::maybe_row_to_map(&row))
                             })
                            .collect()
                    )
                })
        })
}

pub fn select_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<(Beacon, Option<Map>)>), Error=AkError> {
    client
        .prepare("
            SELECT * FROM runtime.beacons
            WHERE b_id = $1::INTEGER
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
                        Some(r) => (client, Some((row_to_beacon(&r), None))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_beacons_for_map(mut client: tokio_postgres::Client, id: Option<i32>) -> impl Future<Item=(tokio_postgres::Client, Vec<Beacon>), Error=AkError> {
    let query = if id.is_some() {
        "
            SELECT * FROM runtime.beacons
            WHERE b_map_id = $1
        "
    } else {
        "
            SELECT * FROM runtime.beacons
            WHERE b_map_id IS NULL
        "
    };

    client
        .prepare_typed(
            query
        , &[
            Type::INT4
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_beacon(&row)).collect())
                })
        })
}

pub fn select_beacons_by_mac(mut client: tokio_postgres::Client, macs: Vec<MacAddress8>) -> impl Future<Item=(tokio_postgres::Client, Vec<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            SELECT * FROM runtime.beacons
            WHERE b_mac_address IN ($1, $2, $3)
        ", &[
            Type::MACADDR8,
            Type::MACADDR8,
            Type::MACADDR8,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[
                    &macs[0],
                    &macs[1],
                    &macs[2],
                ])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_beacon(&row)).collect())
                })
        })
}

pub fn select_beacon_by_mac(mut client: tokio_postgres::Client, mac: MacAddress8) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            SELECT * FROM runtime.beacons
            WHERE b_mac_address = $1
        ", &[
            Type::MACADDR8,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&mac])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_beacon_by_ip(mut client: tokio_postgres::Client, ip: IpAddr) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            SELECT * FROM runtime.beacons
            WHERE b_ip = $1
        ", &[
            Type::INET,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[
                    &ip,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_beacon_prefetch(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<(Beacon, Option<Map>)>), Error=AkError> {
    // TODO paging
    client
        .prepare_typed("
            SELECT *
            FROM runtime.maps AS map, runtime.beacons AS beacon
            WHERE
                map.m_id = beacon.b_map_id
                AND beacon.b_id = $1
        ", &[
            Type::INT4
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
                        Some(r) => (client, Some((row_to_beacon(&r), Some(map::row_to_map(&r))))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            INSERT INTO runtime.beacons (
                b_coordinates,
                b_ip,
                b_last_active,
                b_mac_address,
                b_map_id,
                b_name,
                b_note
            )
            VALUES( $1, $2, $3, $4, $5, $6, $7 )
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INET,
            Type::TIMESTAMPTZ,
            Type::MACADDR8,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let coords = vec![beacon.coordinates[0], beacon.coordinates[1]];
            client
                .query(&statement, &[
                    &coords,
                    &beacon.ip,
                    &beacon.last_active,
                    &beacon.mac_address,
                    &beacon.map_id,
                    &beacon.name,
                    &beacon.note
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    println!("err is {}", err);
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.beacons
            SET
                b_coordinates = $1,
                b_ip = $2,
                b_mac_address = $3,
                b_map_id = $4,
                b_name = $5,
                b_note = $6
             WHERE
                b_id = $7
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INET,
            Type::MACADDR8,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let coords = vec![beacon.coordinates[0], beacon.coordinates[1]];
            client
                .query(&statement, &[
                    &coords,
                    &beacon.ip,
                    &beacon.mac_address,
                    &beacon.map_id,
                    &beacon.name,
                    &beacon.note,
                    &beacon.id,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_beacon_from_realtime(mut client: tokio_postgres::Client, realtime: RealtimeBeacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.beacons
            SET
                b_ip = $1,
                b_last_active = $2,
                b_state = $3
             WHERE
                b_id = $4
            RETURNING *
        ", &[
            Type::INET,
            Type::TIMESTAMPTZ,
            Type::INT2,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[
                    &realtime.ip,
                    &realtime.last_active,
                    &i16::from(realtime.state),
                    &realtime.id,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=AkError> {
    client
        .prepare_typed("
            DELETE FROM runtime.beacons
            WHERE (
                b_id = $1
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
    use std::net::{ IpAddr, Ipv4Addr, };
    use futures::future::join_all;

    #[test]
    fn insert() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
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
    fn delete() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, opt_beacon)| {
                delete_beacon(client, opt_beacon.unwrap().id)
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

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();
        let mut updated_beacon = beacon.clone();
        updated_beacon.name = "hello".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, opt_beacon)| {
                let b = opt_beacon.unwrap();
                updated_beacon.map_id = b.map_id;
                updated_beacon.id = b.id;
                update_beacon(client, updated_beacon)
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
    fn update_from_realtime() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();
        let mut realtime = RealtimeBeacon::from(beacon.clone());
        realtime.last_active = Utc.timestamp(77, 0);

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(move |(client, opt_beacon)| {
                realtime.id = opt_beacon.unwrap().id;
                update_beacon_from_realtime(client, realtime)
            })
            .map(|(_client, opt_beacon)| {
                assert!(opt_beacon.unwrap().last_active == Utc.timestamp(77, 0));
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

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, opt_beacon)| {
                select_beacon(client, opt_beacon.unwrap().id)
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
    fn select_mac() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();
        let mac = MacAddress8::from_bytes(&[1, 0, 0, 0, 0, 0, 0, 0]).unwrap();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();
        beacon.mac_address = mac;

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, _opt_beacon)| {
                select_beacon_by_mac(client, mac)
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
    fn select_for_map() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, Map::new())
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, opt_beacon)| {
                select_beacons_for_map(client, opt_beacon.unwrap().map_id)
            })
            .map(|(_client, _beacons)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select beacons for map");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_prefetch() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, opt_beacon)| {
                select_beacon_prefetch(client, opt_beacon.unwrap().id)
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
    fn select_many() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, _opt_beacon)| {
                select_beacons(client)
            })
            .map(|(_client, _beacons)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_3_by_mac() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacons = vec![Beacon::new(), Beacon::new(), Beacon::new()];
        for (i, mut b) in beacons.iter_mut().enumerate() {
            b.name = i.to_string();
            b.mac_address = MacAddress8::from_bytes(&[i as u8, 0, 0, 0, 0, 0, 0, 0]).unwrap();
            b.ip = IpAddr::V4(Ipv4Addr::new(i as u8, 0, 0, 0));
        }

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, opt_map)| {
                let map_id = opt_map.unwrap().id;
                join_all(beacons
                    .into_iter()
                    .map(move |mut b| {
                        b.map_id = Some(map_id);
                        db_utils::default_connect()
                            .and_then(|client| {
                                insert_beacon(client, b)
                                    .map(|(_client, beacon)| { beacon })
                            })
                    })
                )
                .map(|beacons| {
                    (client, beacons)
                })
            })
            .and_then(|(client, beacons)| {
                let macs: Vec<MacAddress8> = beacons.into_iter().map(|b| b.unwrap().mac_address).collect();
                select_beacons_by_mac(client, macs)
            })
            .map(|(_client, _beacons)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_many_prefetch() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let map = Map::new();

        let mut beacon = Beacon::new();
        beacon.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                // a beacon must point to a valid map
                map::insert_map(client, map)
            })
            .and_then(|(client, map)| {
                beacon.map_id = Some(map.unwrap().id);
                insert_beacon(client, beacon)
            })
            .and_then(|(client, _opt_beacon)| {
                select_beacons_prefetch(client)
            })
            .map(|(_client, _beacons)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }
}
