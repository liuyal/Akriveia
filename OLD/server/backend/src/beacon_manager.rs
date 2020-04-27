
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.

use actix::prelude::*;
use actix_web::Result;
use crate::beacon_udp::*;
use crate::dummy_udp::*;
use crate::data_processor::*;
use crate::db_utils;
use crate::models::network_interface;
use crate::models::beacon;
use std::time::Duration;
use std::collections::{ BTreeSet, BTreeMap, };
use common::*;
use std::net::{ IpAddr, Ipv4Addr, };
use chrono::{ DateTime, Duration as cDuration, };
use crate::ak_error::AkError;

// Problem: Requests to beacons do not create a request object, and so
// we need to manually monitor when beacons do not respond within a given time frame.
// The manager itself has a BeaconState, indicating what the desired beacon's state should be
// for all beacons. This allows us to know what state to tell the beacons to switch to after they
// reboot.
// 1. The manager must ping all beacons in the same state as the manager.
    // The manager will always be either idle or active.
    // ie when a beacon is idle and the manager is idle as well.
    // when the beacon is the same state as the manager, but the beacon does not respond in a
    // timely fashion, then the manager needs to detect this and send a ping request more
    // frequently, up to a limit of 3 repings before the manager sends a reboot request and marks
    // that beacon as "rebooting". if 3 reboot requests fail, then set the beacon to unknown.
// 2. The manager must tell the beacons to switch to the state that the manager is in.
    // again, if the beacon does not respond then resend the requests until 3 repeats have occurred and
    // send the reboot command. if that fails then set the beacon to unknown.
// 3. retries occur more frequently from normal pings or state switches.
// 4. pings always occur at a frequency, however the frequency varies depending on whether or not
    // the system is in idle or active mode.
// 5. a retry will be sent at a time x after the ping/heartbeat requests.

// sln:
// 1. ping beacons every x seconds, where the x value depends on the state. This is a broadcast
    // request.
// 2. on command, send requests to change the state of the beacons. this is a broadcast.
// 3. after y seconds of a request, scan the beacons map to determine if their timestamps have not
    // been updated. (the timestamp should change when a beacon replies). if the timestamp has not
    // been updated, then add them to the stale map, which maps a mac address to a retry count.
// 4. note that a ping request may happen at the same time as a state change request, this is fine
    // as either will update their timestamp.
// 5. as long as the stale map has elements, repeat the retry callback.

const USE_DUMMY_BEACONS: bool = true;
const USE_UDP_BEACONS: bool = false;
const PING_INTERVAL: Duration = Duration::from_millis(100000);
const EMERGENCY_PING_INTERVAL: Duration = Duration::from_millis(10000);
const RESPONSE_THRESHOLD: Duration = Duration::from_millis(2000);
const RETRIES_THRESHOLD: u32 = 4;

#[derive(Debug)]
struct Retries {
    pub expected_response: DateTime<Utc>,
    pub retries: u32,
}

impl Retries {
    fn new() -> Retries {
        Retries {
            expected_response: Utc::now() - cDuration::from_std(RESPONSE_THRESHOLD).unwrap(),
            retries:  0,
        }
    }
}

#[derive(Debug)]
struct BeaconStatus {
    pub realtime: RealtimeBeacon,
    pub retries: Option<Retries>,
}

pub struct BeaconManager {
    state: BeaconState,
    data_processor: Addr<DataProcessor>,
    diagnostic_data: common::DiagnosticData,
    udp_connections: Vec<Addr<BeaconUDP>>,
    dummy_udp_connections: Vec<Addr<DummyUDP>>,
    pinger: Option<SpawnHandle>,
    request_health: Option<SpawnHandle>,
    beacons: BTreeMap<MacAddress8, BeaconStatus>,
    unknown_macs: BTreeSet<MacAddress8>,
}

impl BeaconManager {
    pub fn is_emergency(&self) -> bool {
        return self.state == BeaconState::Active;
    }
}

impl Actor for BeaconManager {
    type Context = Context<Self>;
}

pub enum BMResponse {
    Start(IpAddr, MacAddress8),
    End(IpAddr, MacAddress8),
    Ping(IpAddr, MacAddress8),
    Reboot(IpAddr, MacAddress8),
    TagData(IpAddr, TagData),
    SetIp(IpAddr, MacAddress8),
}

#[derive(Debug, Clone)]
pub enum BMCommand {
    GetEmergency,
    ScanBeacons,
    StartEmergency(Option<MacAddress8>),
    EndEmergency(Option<MacAddress8>),
    Ping(Option<MacAddress8>),
    Reboot(Option<MacAddress8>),
    SetIp(Ipv4Addr),
}

impl Message for BMCommand {
    type Result = Result<bool, AkError>;
}

#[derive(Clone, Copy)]
pub enum BeaconCommand {
    EndEmergency(Option<IpAddr>),
    StartEmergency(Option<IpAddr>),
    Ping(Option<IpAddr>),
    Reboot(Option<IpAddr>),
    SetIp(Ipv4Addr),
}

impl Message for BeaconCommand {
    type Result = Result<(), ()>;
}

impl Message for BMResponse {
    type Result = Result<(), ()>;
}

impl BeaconManager {
    pub fn new(dp: Addr<DataProcessor>) -> Addr<BeaconManager> {
        BeaconManager::create(move |context| {
            let mut manager = BeaconManager {
                state: BeaconState::Idle, // TODO get from db
                data_processor: dp,
                diagnostic_data: common::DiagnosticData::new(),
                udp_connections: Vec::new(),
                dummy_udp_connections: Vec::new(),
                pinger: None,
                request_health: Default::default(),
                unknown_macs: BTreeSet::new(),
                beacons: BTreeMap::new(),
            };
            manager.ping_health(context, PING_INTERVAL);

            let fut = db_utils::default_connect()
                .map_err(AkError::from)
                .and_then(|client| {
                    beacon::select_beacons(client)
                })
                .into_actor(&manager)
                .map(|(_client, beacons), actor, context| {
                    beacons.into_iter().for_each(|b| {
                        actor.beacons.insert(b.mac_address.clone(), BeaconStatus {
                            realtime: RealtimeBeacon::from(b),
                            retries: None,
                        });
                    });
                    context.notify(BMCommand::Ping(None));
                })
                .map_err(|_err, _actor, _context| { });
            context.spawn(fut);

            manager
        })
    }

    fn find_beacon(&mut self, context: &mut Context<Self>, mac: MacAddress8) {
        let dup = mac.clone();
        let fut = db_utils::default_connect()
            .map_err(AkError::from)
            .and_then(move |client| {
                beacon::select_beacon_by_mac(client, dup)
            })
            .into_actor(self)
            .map(move |(_client, beacon), actor, context| {
                if let Some(b) = beacon {
                    actor.beacons.insert(b.mac_address.clone(), BeaconStatus {
                        realtime: RealtimeBeacon::from(b),
                        retries: None,
                    });
                    context.notify(BMCommand::Ping(Some(mac)));
                } else {
                    actor.unknown_macs.insert(mac);
                }
            })
            .map_err(move |_err, actor, _context| {
                actor.unknown_macs.insert(mac);
            });
        context.spawn(fut);
    }

    // this callback is executed only after a request is sent to verify that beacons have responded
    fn check_health(&mut self, context: &mut Context<Self>) {
        let manager_state = self.state;
        let mut any_retries = false;
        self.beacons.iter_mut().for_each(|(_mac, status)| {
            // determine if further action is necessary before the next ping
            let set_none = if let Some(retries) = &mut status.retries {
                retries.retries += 1;
                // this beacon has had a request sent to it recently
                if status.realtime.last_active > retries.expected_response {
                    if manager_state == status.realtime.state {
                        true
                    } else {
                        match manager_state {
                            BeaconState::Idle => context.notify(BMCommand::EndEmergency(Some(status.realtime.mac_address))),
                            BeaconState::Active => context.notify(BMCommand::StartEmergency(Some(status.realtime.mac_address))),
                            _ => panic!("Manager must always be in idle or active states, other states are invalid"),
                        }
                        false
                    }
                    // this beacon has responded, no further action required for now
                } else {
                    any_retries = true;
                    // this beacon has not responded yet, try again
                    if retries.retries > RETRIES_THRESHOLD {
                        if status.realtime.state == BeaconState::Rebooting {
                            // beacon failed to reply and failed to reboot, set to unknown
                            status.realtime.state = BeaconState::Unknown;
                            true
                        } else {
                            // beacon failed to reply, try to reboot it
                            status.realtime.state = BeaconState::Rebooting;
                            retries.retries = 0;
                            context.notify(BMCommand::Reboot(Some(status.realtime.mac_address)));
                            false
                        }
                        // retry a request
                    } else if status.realtime.state == manager_state {
                        // beacon is in the correct state, resend a ping.
                        context.notify(BMCommand::Ping(Some(status.realtime.mac_address)));
                        false
                    } else if status.realtime.state == BeaconState::Rebooting {
                        // dont spam reboots
                        false
                    } else {
                        match manager_state {
                            BeaconState::Idle => context.notify(BMCommand::EndEmergency(Some(status.realtime.mac_address))),
                            BeaconState::Active => context.notify(BMCommand::StartEmergency(Some(status.realtime.mac_address))),
                            _ => panic!("Manager must always be in idle or active states, other states are invalid"),
                        }
                        false
                    }
                }
            } else {
                // beacon has replied, make sure they are in the correct state
                false
            };

            if set_none {
                status.retries = None; // reset
            }
        });

        if any_retries {
            self.request_health = Some(context.run_later(RESPONSE_THRESHOLD, |actor, context| {
                actor.check_health(context);
            }));
        } else {
            self.request_health = None;
        }
    }

    // this callback is executed on a regular basis
    fn ping_health(&mut self, ctx: &mut Context<Self>, dur: Duration) {
        if let Some(pinger) = self.pinger {
            ctx.cancel_future(pinger);
        }
        self.pinger = Some(ctx.run_interval(dur, |actor, context| {
            actor.beacons.iter().for_each(|(_mac, beacon)| {
                let realtime = beacon.realtime.clone();
                let fut = db_utils::default_connect()
                    .map_err(AkError::from)
                    .and_then(|client| {
                        beacon::update_beacon_from_realtime(client, realtime)
                    })
                    .map(|(_client, _beacon)| { })
                    .map_err(|_err| { });
                context.spawn(fut.into_actor(actor));
            });

            context.notify(BMCommand::Ping(None));
        }));
    }

    fn find_beacons(&mut self, context: &mut Context<Self>) {
        if USE_DUMMY_BEACONS { self.find_beacons_dummy(context); }
        if USE_UDP_BEACONS { self.find_beacons_udp(context); }
    }

    fn find_beacons_udp(&mut self, context: &mut Context<Self>) {
        let fut = db_utils::default_connect()
            .and_then(|client| {
                network_interface::select_network_interfaces(client)
                    .map(|(_client, ifaces)| {
                        ifaces
                    })
            })
            .into_actor(self)
            .and_then(|ifaces, actor, context| {
                for iface in ifaces {
                    match iface.beacon_port {
                        Some(port) => {
                            actor.udp_connections.push(BeaconUDP::new(context.address(), iface.ip.clone(), port as u16));
                        },
                        None => {},
                    }
                }
                fut::result(Ok(()))
            })
            .map_err(|err, _, _| {
                println!("failed to create udp connection {}", err);
            });
        context.spawn(fut);
    }

    fn find_beacons_dummy(&mut self, context: &mut Context<Self>) {
        self.dummy_udp_connections.push(DummyUDP::new(context.address()));
    }

    fn mass_send(&self, msg: BeaconCommand) {
        for connection in &self.udp_connections {
            connection.do_send(msg);
        }

        for connection in &self.dummy_udp_connections {
            connection.do_send(msg);
        }
    }
}

impl Handler<BMCommand> for BeaconManager {
    type Result = Result<bool, AkError>;

    fn handle(&mut self, msg: BMCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BMCommand::GetEmergency => { },
            BMCommand::ScanBeacons => {
                self.find_beacons(context);
            },
            BMCommand::StartEmergency(opt_mac) => {
                if self.state != BeaconState::Active {
                    self.state = BeaconState::Active;
                    self.diagnostic_data = common::DiagnosticData::new();
                    self.ping_health(context, EMERGENCY_PING_INTERVAL);
                }

                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get_mut(&mac);
                    if let Some(beacon) = opt_beacon {
                        let ip = beacon.realtime.ip;
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                        self.mass_send(BeaconCommand::StartEmergency(Some(ip)));
                    }
                } else {
                    self.mass_send(BeaconCommand::StartEmergency(None));
                    self.beacons.iter_mut().for_each(|(_mac, beacon)| {
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                    });
                }
            }
            BMCommand::EndEmergency(opt_mac) => {
                if self.state != BeaconState::Idle {
                    self.state = BeaconState::Idle;
                    self.diagnostic_data = common::DiagnosticData::new();
                    self.ping_health(context, PING_INTERVAL);
                }

                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get_mut(&mac);
                    if let Some(beacon) = opt_beacon {
                        let ip = beacon.realtime.ip;
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                        self.mass_send(BeaconCommand::EndEmergency(Some(ip)));
                    }
                } else {
                    self.mass_send(BeaconCommand::EndEmergency(None));
                    self.beacons.iter_mut().for_each(|(_mac, beacon)| {
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                    });
                }
            },
            BMCommand::Ping(opt_mac) => {
                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get_mut(&mac);
                    if let Some(beacon) = opt_beacon {
                        let ip = beacon.realtime.ip;
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                        self.mass_send(BeaconCommand::Ping(Some(ip)));
                    }
                } else {
                    self.mass_send(BeaconCommand::Ping(None));
                    self.beacons.iter_mut().for_each(|(_mac, beacon)| {
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                    });
                }
            },
            BMCommand::Reboot(opt_mac) => {
                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get_mut(&mac);
                    if let Some(beacon) = opt_beacon {
                        let ip = beacon.realtime.ip;
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                        self.mass_send(BeaconCommand::Reboot(Some(ip)));
                    }
                } else {
                    self.mass_send(BeaconCommand::Reboot(None));
                    self.beacons.iter_mut().for_each(|(_mac, beacon)| {
                        if beacon.retries.is_none() {
                            beacon.retries = Some(Retries::new());
                        }
                    });
                }
            },
            BMCommand::SetIp(ip) => {
                self.mass_send(BeaconCommand::SetIp(ip));
                self.beacons.iter_mut().for_each(|(_mac, beacon)| {
                    if beacon.retries.is_none() {
                        beacon.retries = Some(Retries::new());
                    }
                });
            },
        }

        if self.request_health.is_none() {
            self.request_health = Some(context.run_later(RESPONSE_THRESHOLD, |actor, context| {
                actor.check_health(context);
            }));
        }
        Ok(self.is_emergency())
    }
}

impl Handler<BMResponse> for BeaconManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BMResponse, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BMResponse::Start(ip, mac) => {
                match self.beacons.get_mut(&mac) {
                    Some(beacon) => {
                        beacon.realtime.state = BeaconState::Active;
                        beacon.realtime.last_active = Utc::now();
                        beacon.realtime.ip = ip;
                    },
                    None => {
                        self.find_beacon(context, mac);
                    }
                }
            }
            BMResponse::End(ip, mac) => {
                match self.beacons.get_mut(&mac) {
                    Some(beacon) => {
                        beacon.realtime.state = BeaconState::Idle;
                        beacon.realtime.last_active = Utc::now();
                        beacon.realtime.ip = ip;
                    },
                    None => {
                        self.find_beacon(context, mac);
                    }
                }
            },
            BMResponse::Ping(ip, mac) => {
                match self.beacons.get_mut(&mac) {
                    Some(beacon) => {
                        beacon.realtime.last_active = Utc::now();
                        beacon.realtime.ip = ip;
                    },
                    None => {
                        self.find_beacon(context, mac);
                    }
                }
            },
            BMResponse::Reboot(ip, mac) => {
                match self.beacons.get_mut(&mac) {
                    Some(beacon) => {
                        beacon.realtime.state = BeaconState::Rebooting;
                        beacon.realtime.last_active = Utc::now();
                        beacon.realtime.ip = ip;
                    },
                    None => {
                        self.find_beacon(context, mac);
                    }
                }
            },
            BMResponse::SetIp(ip, mac) => {
                match self.beacons.get_mut(&mac) {
                    Some(beacon) => {
                        beacon.realtime.last_active = Utc::now();
                        beacon.realtime.ip = ip;
                    },
                    None => {
                        self.find_beacon(context, mac);
                    }
                }
            },
            BMResponse::TagData(ip, tag_data) => {
                match self.beacons.get_mut(&tag_data.beacon_mac) {
                    Some(beacon) => {
                        if tag_data.tag_distance < 50.0 { // any distance over 50meter is garbage data
                            self.diagnostic_data.tag_data.push(tag_data.clone());
                            self.data_processor.do_send(InLocationData(tag_data));
                            beacon.realtime.ip = ip;
                            beacon.realtime.last_active = Utc::now();
                        }
                    },
                    None => {
                        self.unknown_macs.insert(tag_data.beacon_mac);
                    }
                }
            },

        }
        Ok(())
    }
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, AkError>;
}
impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, AkError>;

    fn handle(&mut self, _msg: GetDiagnosticData, _context: &mut Context<Self>) -> Self::Result {
        let res = self.diagnostic_data.clone();
        self.diagnostic_data.tag_data = Vec::new();
        Ok(res)
    }
}

pub struct OutBeaconData;

impl Message for OutBeaconData {
    type Result = Result<Vec<RealtimeBeacon>, AkError>;
}

impl Handler<OutBeaconData> for BeaconManager {
    type Result = Result<Vec<RealtimeBeacon>, AkError>;

    fn handle (&mut self, _msg: OutBeaconData, _: &mut Context<Self>) -> Self::Result {
        Ok(self.beacons.iter().map(|(_mac, beacon)| beacon.realtime.clone()).collect())
    }
}
