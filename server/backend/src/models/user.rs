use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use crate::ak_error::AkError;

fn row_to_user(row: &Row) -> TrackedUser {
    let mut entry = TrackedUser::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "u_id" => entry.id = row.get(i),
            "u_coordinates" => {
                let coords: Vec<f64> = row.get(i);
                entry.coordinates = na::Vector2::new(coords[0], coords[1]);
            },
            "u_attached_user" => entry.attached_user = row.get(i),
            "u_employee_id" => entry.employee_id = row.get(i),
            "u_last_active" => entry.last_active = row.get(i),
            "u_mac_address" => {
                let short: Option<i16> = row.get(i);
                entry.mac_address = short.map(|m| ShortAddress::from_pg(m));
            },
            "u_map_id" => entry.map_id = row.get(i),
            "u_name" => entry.name = row.get(i),
            "u_note" => entry.note = row.get(i),
            "u_work_phone" => entry.work_phone = row.get(i),
            "u_mobile_phone" => entry.mobile_phone = row.get(i),
            unhandled if unhandled.starts_with("u_") => { panic!("unhandled user column {}", unhandled); },
            _ => {},
        }
    }
    entry
}

pub fn select_users(mut client: tokio_postgres::Client, include_contacts: bool) -> impl Future<Item=(tokio_postgres::Client, Vec<TrackedUser>), Error=AkError> {
    // TODO paging
    let query = if include_contacts {
        "SELECT * FROM runtime.users"
    } else {
        "
            SELECT * FROM runtime.users
            WHERE u_attached_user IS NULL
        "
    };

    client
        .prepare(query)
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map_err(AkError::from)
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_user(&row)).collect())
                })
        })
}

pub fn select_user(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>, Option<TrackedUser>), Error=AkError> {
    client
        .prepare("
            SELECT * FROM runtime.users
            WHERE u_id = $1::INTEGER
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
                        Some(r) => (client, Some(row_to_user(&r)), None),
                        _ => (client, None, None),
                    }
                })
        })
}

pub fn select_user_by_short(mut client: tokio_postgres::Client, id: ShortAddress) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare("
            SELECT * FROM runtime.users
            WHERE u_mac_address = $1
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[&id.as_pg()])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_by_attached_user(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare("
            SELECT * FROM runtime.users
            WHERE u_attached_user = $1::INTEGER
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
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_user_prefetch(client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>, Option<TrackedUser>), Error=AkError> {
    select_user(client, id)
        .and_then(move |(client, opt_user, _)| {
            select_by_attached_user(client, id)
                .map(move |(client, opt_contact)| {
                    (client, opt_user, opt_contact)
                })
        })
}

pub fn select_user_random(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare("
            SELECT *
            FROM runtime.users
            WHERE u_mac_address IS NOT NULL
            ORDER BY random()
            LIMIT 1
        ")
        .map_err(AkError::from)
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_user(mut client: tokio_postgres::Client, user: TrackedUser) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare_typed("
            INSERT INTO runtime.users (
                u_coordinates,
                u_attached_user,
                u_employee_id,
                u_last_active,
                u_mac_address,
                u_map_id,
                u_name,
                u_note,
                u_work_phone,
                u_mobile_phone
            )
            VALUES( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::TIMESTAMPTZ,
            Type::INT2,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let coordinates = vec![user.coordinates[0], user.coordinates[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &user.attached_user,
                    &user.employee_id,
                    &user.last_active,
                    &user.mac_address.map(|m| m.as_pg()),
                    &user.map_id,
                    &user.name,
                    &user.note,
                    &user.work_phone,
                    &user.mobile_phone,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_user(mut client: tokio_postgres::Client, user: TrackedUser) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.users
            SET
                u_coordinates = $1,
                u_attached_user = $2,
                u_employee_id = $3,
                u_last_active = $4,
                u_mac_address = $5,
                u_map_id = $6,
                u_name = $7,
                u_note = $8,
                u_work_phone = $9,
                u_mobile_phone = $10
             WHERE
                u_id = $11
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::TIMESTAMPTZ,
            Type::INT2,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::INT4,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let coordinates = vec![user.coordinates[0], user.coordinates[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &user.attached_user,
                    &user.employee_id,
                    &user.last_active,
                    &user.mac_address.map(|m| m.as_pg()),
                    &user.map_id,
                    &user.name,
                    &user.note,
                    &user.work_phone,
                    &user.mobile_phone,
                    &user.id,
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_user_from_realtime(mut client: tokio_postgres::Client, realtime: RealtimeUserData) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=AkError> {
    client
        .prepare_typed("
            UPDATE runtime.users
            SET
                u_coordinates = $1,
                u_last_active = $2,
                u_map_id = $3
             WHERE
                u_mac_address = $4
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::TIMESTAMPTZ,
            Type::INT4,
            Type::INT2,
        ])
        .map_err(AkError::from)
        .and_then(move |statement| {
            let coordinates = vec![realtime.coordinates[0], realtime.coordinates[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &realtime.last_active,
                    &realtime.map_id,
                    &realtime.addr.as_pg(),
                ])
                .into_future()
                .map_err(|(err, _next)| {
                    AkError::from(err)
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_user(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=AkError> {
    client
        .prepare_typed("
            DELETE FROM runtime.users
            WHERE (
                u_id = $1
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

        let user = TrackedUser::new();

        let task = db_utils::default_connect()
            .and_then(|client| {
                let expected = user.clone();
                insert_user(client, user)
                    .map(move |(_client, opt_user)| {
                        assert!(opt_user.unwrap().mac_address == expected.mac_address);
                    })
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();
        let mut updated_user = user.clone();
        updated_user.name = "user_1".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                updated_user.id = opt_user.unwrap().id;
                update_user(client, updated_user)
            })
            .map(|(_client, _user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update_from_realtime() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();
        user.mac_address = Some(ShortAddress::from_bytes(&[0, 3]).unwrap());

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                let mut realtime = RealtimeUserData::from(opt_user.unwrap().clone());
                realtime.coordinates = na::Vector2::new(0.5, 0.5);
                update_user_from_realtime(client, realtime)
            })
            .map(|(_client, user)| {
                assert!(user.unwrap().coordinates == na::Vector2::new(0.5, 0.5));
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert users");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                select_user(client, opt_user.unwrap().id)
            })
            .map(|(_client, _opt_user, _opt_e_user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_by_short() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();
        user.mac_address = Some(ShortAddress::from_bytes(&[0, 3]).unwrap());

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                select_user_by_short(client, opt_user.unwrap().mac_address.unwrap())
            })
            .map(|(_client, _opt_user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_prefetch() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let mut e_user = TrackedUser::new();
        e_user.name = "user_1".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                e_user.attached_user = Some(opt_user.unwrap().id);
                insert_user(client, e_user)
            })
            .and_then(|(client, opt_user)| {
                select_user_prefetch(client, opt_user.unwrap().id)
            })
            .map(|(_client, _opt_user, _opt_e_user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_attached_user() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let mut contact = TrackedUser::new();
        contact.name = "contact_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                let id = opt_user.as_ref().unwrap().id;
                contact.attached_user = Some(id);
                insert_user(client, contact)
                    .map(move |(client, opt_contact)| {
                        (client, opt_user, opt_contact)
                    })
            })
            .and_then(|(client, opt_user, opt_contact)| {
                select_by_attached_user(client, opt_user.as_ref().unwrap().id)
                    .map(|(client, should_be_opt_contact)| {
                        assert!(should_be_opt_contact.as_ref().unwrap().id == opt_contact.as_ref().unwrap().id);
                        (client, opt_user, opt_contact)
                    })
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select multiple users");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_random() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, _opt_user)| {
                select_user_random(client)
            })
            .map(|(_client, opt_user)| {
                assert!(opt_user.is_some());
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_many() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, _opt_user)| {
                select_users(client, true)
            })
            .map(|(_client, _users)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select multiple users");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_many_include_contacts() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let mut contact = TrackedUser::new();
        contact.name = "contact_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                contact.attached_user = Some(opt_user.unwrap().id);
                insert_user(client, contact)
            })
            .and_then(|(client, _opt_user)| {
                select_users(client, true)
            })
            .and_then(|(client, users)| {
                assert!(users.iter().find(|u| u.attached_user.is_some()).is_some());
                assert!(users.iter().find(|u| u.attached_user.is_none()).is_some());
                select_users(client, false)
            })
            .map(|(_client, users)| {
                assert!(users.iter().find(|u| u.attached_user.is_some()).is_none());
                assert!(users.iter().find(|u| u.attached_user.is_none()).is_some());
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select multiple users");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn delete() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db(true)).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                delete_user(client, opt_user.unwrap().id)
            })
            .map(|_client| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to delete user");
            });
        runtime.block_on(task).unwrap();
    }
}



