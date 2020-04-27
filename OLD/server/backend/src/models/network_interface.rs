
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use ipnet::Ipv4Net;
use std::net::{ IpAddr, Ipv4Addr, };
use crate::ak_error::AkError;

// NOTE:
// The rust std type IpAddr does not support subnet masks, but does support postgres
// (de)serialization; while the Ipv4Net type does support subnet masks, but is not supported by
// tokio postgres (de)serialization. To solve this, store the subnet manually in postgres(even
// though the INET type does support subnets, it is ignored due to no built in access to it)
// and merge the subnet mask with the ip to create the Ipv4Net object for its beneficial built in
// manipulations and helper functions. In addition to this, postgres also does not support unsigned
// types, so the mask is stored in postgres with 2 signed bytes rather than 1 unsigned byte. ugh.

pub fn row_to_network_interface(row: &Row) -> NetworkInterface {
    let mut obj = NetworkInterface::new();
    let mut ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let mut mask: u8 = 0;
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "n_id" => obj.id = row.get(i),
            "n_beacon_port" => obj.beacon_port = row.get(i),
            "n_ip" => ip = row.get(i),
            "n_mask" => mask = row.get::<usize, i16>(i) as u8,
            "n_mac" => obj.mac = row.get(i),
            "n_name" => obj.name = row.get(i),
            "n_webserver_port" => obj.webserver_port = row.get(i),
            unhandled if unhandled.starts_with("n_") => { panic!("unhandled network interface column {}", unhandled); },
            _ => {},
        }
    }
    let ip4 = match ip {
        IpAddr::V4(inner) => inner,
        IpAddr::V6(_) => panic!("ipv6 not supported"),
    };
    obj.ip = Ipv4Net::new(ip4, mask).unwrap();
    obj
}

pub fn select_network_interfaces(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<NetworkInterface>), Error=AkError> {
    // TODO paging
    client
        .prepare("
            SELECT *
            FROM system.network_interfaces
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_network_interface(&row)).collect())
                })
        })
}

pub fn select_network_interface(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<NetworkInterface>), Error=AkError> {
    client
        .prepare("
            SELECT *
            FROM system.network_interfaces
            WHERE n_id = $1::INTEGER
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
                        Some(r) => (client, Some(row_to_network_interface(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_network_interface(mut client: tokio_postgres::Client, iface: NetworkInterface) -> impl Future<Item=(tokio_postgres::Client, Option<NetworkInterface>), Error=AkError> {
    client
        .prepare_typed("
            INSERT INTO system.network_interfaces (
                n_beacon_port,
                n_ip,
                n_mac,
                n_mask,
                n_name,
                n_webserver_port
            )
            VALUES( $1, cast($2 AS INET), $3, $4, $5, $6 )
            RETURNING *
        ", &[
            Type::INT2,
            Type::TEXT,
            Type::MACADDR,
            Type::INT2,
            Type::VARCHAR,
            Type::INT2,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let inet = format!("{}", iface.ip.addr());
            let mask = iface.ip.prefix_len() as i16;
            client
                .query(&statement, &[
                    &iface.beacon_port,
                    &inet,
                    &iface.mac,
                    &mask,
                    &iface.name,
                    &iface.webserver_port,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_network_interface(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_network_interface(mut client: tokio_postgres::Client, iface: NetworkInterface) -> impl Future<Item=(tokio_postgres::Client, Option<NetworkInterface>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE system.network_interfaces
            SET
                n_beacon_port = $1,
                n_ip = CAST($2 AS INET),
                n_mac = $3,
                n_mask = $4,
                n_name = $5,
                n_webserver_port = $6
             WHERE
                n_id = $7
            RETURNING *
        ", &[
            Type::INT2,
            Type::TEXT,
            Type::MACADDR,
            Type::INT2,
            Type::VARCHAR,
            Type::INT2,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let inet = format!("{}", iface.ip.addr());
            let mask = iface.ip.prefix_len() as i16;
            client
                .query(&statement, &[
                    &iface.beacon_port,
                    &inet,
                    &iface.mac,
                    &mask,
                    &iface.name,
                    &iface.webserver_port,
                    &iface.id,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_network_interface(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_network_interface(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=AkError> {
    client
        .prepare_typed("
            DELETE FROM system.network_interfaces
            WHERE (
                n_id = $1
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

        let mut iface = NetworkInterface::new();
        iface.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_network_interface(client, iface)
            })
            .map(|(_client, _iface)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert iface");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn delete() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut iface = NetworkInterface::new();
        iface.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_network_interface(client, iface)
            })
            .and_then(|(client, opt_iface)| {
                delete_network_interface(client, opt_iface.unwrap().id)
            })
            .map(|_client| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert iface");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut iface = NetworkInterface::new();
        iface.name = "hello_test".to_string();
        let mut updated_iface = iface.clone();
        updated_iface.name = "hello".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_network_interface(client, iface)
            })
            .and_then(|(client, opt_iface)| {
                let b = opt_iface.unwrap();
                updated_iface.id = b.id;
                update_network_interface(client, updated_iface)
            })
            .map(|(_client, _iface)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert iface");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut iface = NetworkInterface::new();
        iface.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_network_interface(client, iface)
            })
            .and_then(|(client, opt_iface)| {
                select_network_interface(client, opt_iface.unwrap().id)
            })
            .map(|(_client, _iface)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert iface");
            });
        runtime.block_on(task).unwrap();
    }

#[test]
    fn select_many() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut iface = NetworkInterface::new();
        iface.name = "hello_test".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_network_interface(client, iface)
            })
            .and_then(|(client, _opt_iface)| {
                select_network_interfaces(client)
            })
            .map(|(_client, _ifaces)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert iface");
            });
        runtime.block_on(task).unwrap();
    }
}
