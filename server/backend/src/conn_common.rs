
use common::*;
use crate::beacon_manager::BMResponse;
use regex::Regex;
use std::error::Error;
use std::fmt;
use chrono::Utc;
use std::net::IpAddr;

#[derive(Debug)]
pub enum MessageError {
    ParseFormat,
    ParseFloat,
    ParseMac,
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::ParseFormat => write!(f, "Invalid message format"),
            MessageError::ParseFloat => write!(f, "Failed to parse float"),
            MessageError::ParseMac => write!(f, "Failed to parse mac address"),
        }
    }
}

impl Error for MessageError {
     fn cause(&self) -> Option<&dyn Error> {
         None
     }
 }

impl From<eui64::ParseError> for MessageError {
    fn from(_item: eui64::ParseError) -> Self {
        MessageError::ParseMac
    }
}

impl From<common::short_address::ParseError> for MessageError {
    fn from(_item: common::short_address::ParseError) -> Self {
        MessageError::ParseMac
    }
}

impl From<std::num::ParseFloatError> for MessageError {
    fn from(_item: std::num::ParseFloatError) -> Self {
        MessageError::ParseFloat
    }
}

pub fn parse_message(message: &str, source_ip: IpAddr) -> Result<BMResponse, MessageError> {
    let brack_start = message.find('[');
    let brack_end = message.find(']');
    if let Some((start, end)) = brack_start.and_then(|start| brack_end.map(|end| (start, end))) {
        let slice = &message[start+1..end];
        let split: Vec<&str> = slice.split("|").collect();
        if split.len() >= 2 {
            let beacon_mac = MacAddress8::parse_str(split[0])?;
            let command_type = split[1];
            match command_type {
                "start_ack" => {
                    Ok(BMResponse::Start(source_ip, beacon_mac))
                },
                "end_ack" => {
                    Ok(BMResponse::End(source_ip, beacon_mac))
                },
                "ping_ack" => {
                    Ok(BMResponse::Ping(source_ip, beacon_mac))
                },
                "reboot_ack" => {
                    Ok(BMResponse::Reboot(source_ip, beacon_mac))
                },
                "range_ack" => {
                    println!("short address is: {}", split[2]);
                    let tag_mac = ShortAddress::parse_str(split[2])?;
                    let distance = split[3];
                    let reg = Regex::new(r"/[^$0-9]+/").unwrap();
                    let distance_stripped = reg.replace_all(&distance, "");

                    // remove the last character every time, idk why but there is always
                    // a newline at the end of rssi_stripped. from_str_radix REQUIRES
                    // all numeric characters.
                    if distance_stripped.len() <= 0 {
                        return Err(MessageError::ParseFormat)
                    }
                    let distance_numeric = distance_stripped[..distance_stripped.len() - 1].parse::<f64>()?;

                    Ok(BMResponse::TagData(source_ip, common::TagData {
                        beacon_mac: beacon_mac.clone(),
                        tag_distance: distance_numeric,
                        tag_mac,
                        timestamp: Utc::now(),
                    }))
                },
                _ => {
                    println!("unkown command");
                    Err(MessageError::ParseFormat)
                },
            }
        } else {
            // invalid command
            Err(MessageError::ParseFormat)
        }
    } else {
        // incomplete transmission
        Err(MessageError::ParseFormat)
    }
}
