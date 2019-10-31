extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use rand::{ Rng, SeedableRng, };
use rand::rngs::SmallRng;
use actix::prelude::*;
use actix::{ Actor, Context, };
use crate::db_utils;
use crate::models::user;
use crate::models::beacon;
use crate::beacon_manager::*;
use common::*;
use std::time::Duration;
use futures::future as fut;
use actix::fut as afut;

const MESSAGE_INTERVAL: Duration = Duration::from_millis(1000);
const MIN_DISTANCE: f64 = 1.0;
const MAX_DISTANCE: f64 = 4.0;

pub struct DummyUDP {
    manager: Addr<BeaconManager>,
    data_task: SpawnHandle,
    rng: SmallRng,
}

impl Actor for DummyUDP {
    type Context = Context<Self>;
}

struct GenTagData { }
impl Message for GenTagData {
    type Result = Result<(), ()>;
}

impl Handler<GenTagData> for DummyUDP {
    type Result = ResponseActFuture<Self, (), ()>;

    fn handle(&mut self, _msg: GenTagData, _: &mut Context<Self>) -> Self::Result {
        let b_fut = db_utils::default_connect()
            .and_then(|client| {
                beacon::select_beacons(client)
            });
        let u_fut = db_utils::default_connect()
            .and_then(|client| {
                user::select_user_random(client)
            });

        let data_gen_fut = b_fut.join(u_fut)
            .into_actor(self)
            .and_then(move |((_client1, beacons), (_client2, opt_user)), actor, _context| {
                let time = Utc::now();
                if let Some(user) = opt_user {
                    for b in beacons {
                        let user_distance = actor.rng.gen_range(MIN_DISTANCE, MAX_DISTANCE);
                        actor.manager
                            .do_send( TagDataMessage {
                                data: common::TagData {
                                    beacon_mac: b.mac_address,
                                    tag_distance: user_distance,
                                    tag_mac: user.mac_address.unwrap(),
                                    timestamp: time,
                                }
                            });
                    }
                }
                afut::result(Ok(()))
            })
            .map_err(|_err, _actor, _context| {
            });

        Box::new(data_gen_fut)
    }
}

impl Handler<BeaconCommand> for DummyUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BeaconCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BeaconCommand::StartEmergency(_opt_ip) => {
                self.data_task = context.run_interval(MESSAGE_INTERVAL, |_actor, context| {
                    context.notify(GenTagData{});
                });
            },
            BeaconCommand::EndEmergency(_opt_ip) => {
                context.cancel_future(self.data_task);
            },
            BeaconCommand::Ping(opt_ip) => {
                let beacons_fut = db_utils::default_connect()
                    .and_then(move |client| {
                        if let Some(ip) = opt_ip {
                            fut::Either::B(beacon::select_beacon_by_ip(client, ip)
                                .map(|(client, opt_beacon)| {
                                    if let Some(beacon) = opt_beacon {
                                        (client, vec![beacon])
                                    } else {
                                        (client, Vec::new())
                                    }
                                })
                            )
                        } else {
                            fut::Either::A(beacon::select_beacons(client))
                        }
                    })
                    .into_actor(self)
                    .and_then(move |(_client, beacons), actor, _context| {
                        for b in beacons {
                            actor.manager
                                .do_send(BMResponse::Ping(b.ip, b.mac_address));
                        }
                        afut::result(Ok(()))
                    })
                    .map_err(|_err, _actor, _context| {
                    });
                context.spawn(beacons_fut);
                println!("udp dummy ping");
            },
            _ => {
            }
        }

        Ok(())
    }
}

impl DummyUDP {
    pub fn new(manager: Addr<BeaconManager>) -> Addr<DummyUDP> {
        DummyUDP::create(move |_context| {
            DummyUDP {
                manager,
                rng: SmallRng::from_entropy(),
                data_task: Default::default(),
            }
        })
    }
}
