extern crate serde_derive;
extern crate nalgebra as na;
extern crate eui48;
extern crate eui64;
extern crate ipnet;

pub mod short_address;

pub use chrono::offset::TimeZone;
pub use chrono::{ DateTime, Utc, format::DelayedFormat, format::StrftimeItems, };
pub use eui48::MacAddress;
pub use eui64::MacAddress8;
pub use short_address::ShortAddress;
use ipnet::{ Ipv4Net, };
use serde_derive::{ Deserialize, Serialize, };
use std::net::{ IpAddr, Ipv4Addr, };
use std::fmt;

pub fn beacon_url(id: &str) -> String {
    return format!("/beacon/{}", id);
}
pub fn beacon_command_url() -> String {
    return String::from("/beacons/command");
}
pub fn beacons_url() -> String {
    return String::from("/beacons");
}
pub fn beacons_status_url() -> String {
    return String::from("/beacons/status");
}
pub fn beacons_for_map_url(id: &str) -> String {
    return format!("/map/{}/beacons", id);
}

pub fn user_url(id: &str) -> String {
    return format!("/user/{}", id);
}
pub fn users_url() -> String {
    return String::from("/users");
}
pub fn users_status_url() -> String {
    return String::from("/users/status");
}

pub fn map_url(id: &str) -> String {
    return format!("/map/{}", id);
}
pub fn map_blueprint_url(id: &str) -> String {
    return format!("/map/{}/blueprint", id);
}
pub fn maps_url() -> String {
    return String::from("/maps");
}

pub fn network_url(id: &str) -> String {
    return format!("/network/{}", id);
}
pub fn networks_url() -> String {
    return String::from("/networks");
}

pub fn system_emergency_url() -> String {
    return String::from("/system/emergency");
}

pub fn system_diagnostics_url() -> String {
    return String::from("/system/diagnostics");
}
pub fn system_restart_url() -> String {
    return String::from("/system/restart");
}
pub fn system_ping_url() -> String {
    return String::from("/system/ping");
}

pub fn session_login_url() -> String {
    return String::from("/session/login");
}

pub fn session_logout_url() -> String {
    return String::from("/session/logout");
}

pub fn session_check_url() -> String {
    return String::from("/session/check");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCommandResponse {
    pub emergency: bool,
}

impl SystemCommandResponse {
    pub fn new(emergency: bool) -> SystemCommandResponse {
        SystemCommandResponse {
            emergency,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    pub beacon_mac: MacAddress8,
    pub tag_distance: f64,
    pub tag_mac: ShortAddress,
    pub timestamp: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub tag_data: Vec<TagData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeaconRequest {
    StartEmergency(Option<MacAddress8>),
    EndEmergency(Option<MacAddress8>),
    Ping(Option<MacAddress8>),
    Reboot(Option<MacAddress8>),
    SetIp(Ipv4Addr),
}

impl Default for BeaconRequest {
    fn default() -> Self {
        BeaconRequest::Ping(None)
    }
}

impl DiagnosticData {
    pub fn new() -> DiagnosticData {
        DiagnosticData {
            tag_data: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconTOFToUser {
    pub name: String,
    pub location: na::Vector2<f64>,
    pub distance_to_tag: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeUserData {
    pub addr: ShortAddress,
    pub beacon_tofs: Vec<BeaconTOFToUser>,
    pub coordinates: na::Vector2<f64>,
    pub id: i32,
    pub last_active: DateTime<Utc>,
    pub map_id: Option<i32>,
    pub name: String,
}

impl From<TrackedUser> for RealtimeUserData {
    fn from(user: TrackedUser) -> Self {
        RealtimeUserData {
            addr: user.mac_address.unwrap(), // user must have a mac address to be tracked
            beacon_tofs: Vec::new(),
            coordinates: user.coordinates,
            id: user.id,
            last_active: user.last_active,
            map_id: user.map_id,
            name: user.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedUser {
    pub id: i32,
    pub coordinates: na::Vector2<f64>,
    pub attached_user: Option<i32>,
    pub employee_id: Option<String>,
    pub last_active: DateTime<Utc>,
    pub mac_address: Option<ShortAddress>,
    pub map_id: Option<i32>,
    pub name: String,
    pub note: Option<String>,
    pub work_phone: Option<String>,
    pub mobile_phone: Option<String>,
}

impl TrackedUser {
    pub fn new() -> TrackedUser {
        TrackedUser {
            id: -1, // primary key
            coordinates: na::Vector2::new(0.0, 0.0),
            attached_user: None,
            employee_id: None,
            last_active: Utc.timestamp(0, 0),
            mac_address: None,
            map_id: None,
            name: String::new(),
            note: None,
            work_phone: None,
            mobile_phone: None,
        }
    }

    pub fn merge(&mut self, rt: RealtimeUserData) -> Vec<BeaconTOFToUser> {
        assert!(self.id == rt.id);
        assert!(self.mac_address.unwrap() == rt.addr); // TODO handle this more gracefully

        self.coordinates = rt.coordinates;
        self.last_active = rt.last_active;
        self.map_id = rt.map_id;

        rt.beacon_tofs
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeaconState {
    Unknown,
    Idle,
    Rebooting,
    Active,
}

impl BeaconState {
    pub const fn count() -> usize {
        4
    }
}

impl fmt::Display for BeaconState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BeaconState::Unknown => write!(f, "Unknown"),
            BeaconState::Idle => write!(f, "Idle"),
            BeaconState::Rebooting => write!(f, "Rebooting"),
            BeaconState::Active => write!(f, "Active"),
        }
    }
}

impl From<BeaconState> for usize {
    fn from(s: BeaconState) -> Self {
        // NOTE: When updating this match statement,
        // remember to update the count() function as
        // well.
        match s {
            BeaconState::Unknown     => 0,
            BeaconState::Idle        => 1,
            BeaconState::Rebooting   => 2,
            BeaconState::Active      => 3,
        }
    }
}

impl From<BeaconState> for i16 {
    fn from(s: BeaconState) -> Self {
        usize::from(s) as i16
    }
}

impl From<i16> for BeaconState {
    fn from(s: i16) -> Self {
        // NOTE: When updating this match statement,
        // remember to update the count() function as
        // well.
        match s {
            0 => BeaconState::Unknown,
            1 => BeaconState::Idle,
            2 => BeaconState::Rebooting,
            3 => BeaconState::Active,
            _ => panic!("unexpected beacon state"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub coordinates: na::Vector2<f64>,
    pub id: i32, // primary key
    pub ip: IpAddr,
    pub last_active: DateTime<Utc>,
    pub mac_address: MacAddress8,
    pub map_id: Option<i32>,
    pub name: String,
    pub note: Option<String>,
    pub state: BeaconState,
}

impl Beacon {
    pub fn new() -> Beacon {
        Beacon {
            id: -1,
            coordinates: na::Vector2::new(0.0, 0.0),
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            last_active: Utc.timestamp(0, 0),
            mac_address: MacAddress8::nil(),
            map_id: None,
            name: String::new(),
            note: None,
            state: BeaconState::Unknown,
        }
    }

    pub fn merge(&mut self, rt: RealtimeBeacon) {
        assert!(self.id == rt.id);
        assert!(self.mac_address == rt.mac_address); // TODO handle this more gracefully

        self.last_active = rt.last_active;
        self.state = rt.state;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBeacon {
    pub id: i32, // primary key
    pub ip: IpAddr,
    pub last_active: DateTime<Utc>,
    pub mac_address: MacAddress8,
    pub state: BeaconState,
    pub map_id: Option<i32>,
}

impl From<Beacon> for RealtimeBeacon {
    fn from(beacon: Beacon) -> Self {
        RealtimeBeacon {
            id: beacon.id,
            ip: beacon.ip,
            mac_address: beacon.mac_address,
            last_active: beacon.last_active,
            state: beacon.state,
            map_id: beacon.map_id,
        }
    }
}

impl RealtimeBeacon {
    pub fn new() -> RealtimeBeacon {
        RealtimeBeacon {
            id: -1,
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            last_active: Utc.timestamp(0, 0),
            mac_address: MacAddress8::nil(),
            state: BeaconState::Unknown,
            map_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub id: i32, // primary key
    pub blueprint: Vec<u8>,
    pub bounds: na::Vector2<i32>,
    pub name: String,
    pub note: Option<String>,
    pub scale: f64, // pixels per meter
}

impl Map {
    pub fn new() -> Map {
        Map {
            id: -1,
            blueprint: Vec::new(),
            bounds: na::Vector2::new(0, 0),
            name: String::new(),
            note: None,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: i32, // primary key
    pub beacon_port: Option<i16>,
    pub ip: Ipv4Net,
    pub mac: MacAddress,
    pub name: String,
    pub webserver_port: Option<i16>,
}

impl NetworkInterface {
    pub fn new() -> NetworkInterface {
        NetworkInterface {
            id: -1,
            beacon_port: None,
            ip: Ipv4Net::new(Ipv4Addr::new(0, 0, 0, 0), 32).unwrap(),
            mac: MacAddress::nil(),
            name: String::new(),
            webserver_port: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub name: String,
    pub pw: String,
}

impl LoginInfo {
    pub fn new() -> LoginInfo {
        LoginInfo {
            name: String::new(),
            pw: String::new(),
        }
    }

    pub fn reset_pw(&mut self) {
        self.pw = String::new();
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    StartNormal,
    RebuildDB,
    RebuildDemoDB,
}

// MUST MATCH AkError
#[derive(Clone, Serialize, Deserialize)]
pub struct WebError {
    pub reason: String,
    pub t: AkErrorType,
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum AkErrorType {
    Internal,
    NotFound,
    BadRequest,
    Unauthorized,
    Validation,
    FileUpload,
    ConnectionError,
}

