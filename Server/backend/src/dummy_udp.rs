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
use std::net::IpAddr;
use futures::future as fut;
use actix::fut as afut;

const MESSAGE_INTERVAL: Duration = Duration::from_millis(1000);
const MIN_DISTANCE: f64 = 1.0;
const MAX_DISTANCE: f64 = 4.0;
const REBOOT_AWAKE_CHANCE: f64 = 0.05;
const REBOOT_CHANCE: f64 = 0.0;

pub struct DummyUDP {
    manager: Addr<BeaconManager>,
    data_task: SpawnHandle,
    rng: SmallRng,
    rebooting_ip: Option<IpAddr>,
}

impl Actor for DummyUDP {
    type Context = Context<Self>;
}

struct GenTagData;
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
                        if let Some(ip) = actor.rebooting_ip {
                            if ip == b.ip {
                                // pretend to do nothing while this beacon is "rebooting"
                                continue;
                            }
                            // randomly choose this beacon to be unresponsive
                        } else if actor.rng.gen_bool(REBOOT_CHANCE) {
                            actor.rebooting_ip = Some(b.ip);
                            continue;
                        }

                        let user_distance = actor.rng.gen_range(MIN_DISTANCE, MAX_DISTANCE);
                        actor.manager
                            .do_send(BMResponse::TagData(b.ip, TagData {
                                beacon_mac: b.mac_address,
                                tag_distance: user_distance,
                                tag_mac: user.mac_address.unwrap(),
                                timestamp: time,
                            }));
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
            BeaconCommand::StartEmergency(opt_ip) => {
                self.data_task = context.run_interval(MESSAGE_INTERVAL, |_actor, context| {
                    context.notify(GenTagData);
                });

                self.reply(context, opt_ip, |ip, mac| BMResponse::Start(ip, mac));
            },
            BeaconCommand::EndEmergency(opt_ip) => {
                context.cancel_future(self.data_task);
                self.reply(context, opt_ip, |ip, mac| BMResponse::End(ip, mac));
            },
            BeaconCommand::Ping(opt_ip) => {
                self.reply(context, opt_ip, |ip, mac| BMResponse::Ping(ip, mac));
            },
            BeaconCommand::Reboot(opt_ip) => {
                self.reply(context, opt_ip, |ip, mac| BMResponse::Reboot(ip, mac));
            }
            BeaconCommand::SetIp(_ip) => {
                self.reply(context, None, |ip, mac| BMResponse::SetIp(ip, mac));
            }
        }

        Ok(())
    }
}

impl DummyUDP {
    pub fn new(manager: Addr<BeaconManager>) -> Addr<DummyUDP> {
        DummyUDP::create(move |_context| {
            println!("starting dummy udp actor");
            DummyUDP {
                rebooting_ip: None,
                manager,
                rng: SmallRng::from_entropy(),
                data_task: Default::default(),
            }
        })
    }

    fn reply<F: 'static>(&mut self, context: &mut Context<Self>, opt_ip: Option<IpAddr>, msg: F)
        where F: Fn(IpAddr, MacAddress8) -> BMResponse
    {
        if opt_ip == self.rebooting_ip && self.rng.gen_bool(REBOOT_AWAKE_CHANCE) {
            self.rebooting_ip = None;
        }

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
                    // test out the retry logic by making successfully rebooting a chance.
                    if Some(b.ip) != actor.rebooting_ip {
                        actor.manager
                            .do_send(msg(b.ip, b.mac_address));
                    }
                }
                afut::result(Ok(()))
            })
            .map_err(|_err, _actor, _context| {
            });
        context.spawn(beacons_fut);
    }

}
