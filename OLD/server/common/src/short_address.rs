use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::error::Error;
use std::str::FromStr;

// Credit to https://github.com/abaumhauer/eui48, the short address struct
// defined here is virtually the same as an eui48, except the short address
// only accept 2 bytes rather than 6.

#[derive(Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ShortAddress {
    pub addr: [u8; 2],
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Ord, PartialOrd, Hash)]
pub enum ParseError {
    /// Length is incorrect (should be 5 or 6)
    InvalidLength(usize),
    /// Character not [0-9a-fA-F]|'x'|'-'|':'|'.'
    InvalidCharacter(char, usize),
}

impl ShortAddress {
    pub fn new(bytes: [u8; 2]) -> Self {
        ShortAddress { addr: bytes }
    }

    pub fn nil() -> Self {
        ShortAddress { addr: [0; 2] }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02x}:{:02x}",
            self.addr[0], self.addr[1]
        )
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() != 2 {
            return Err(());
        }
        let mut input: [u8; 2] = Default::default();
        for i in 0..2 {
            input[i] = bytes[i];
        }
        Ok(Self::new(input))
    }

    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.addr
    }

    pub fn as_pg(&self) -> i16 {
        let res = unsafe {
            std::mem::transmute::<[u8; 2], i16>(self.addr)
        };
        res
    }

    pub fn from_pg(addr: i16) -> ShortAddress {
        let res = unsafe {
            std::mem::transmute::<i16, [u8; 2]>(addr)
        };
        ShortAddress {
            addr: res,
        }
    }

    pub fn to_array(&self) -> [u8; 2] {
        self.addr
    }

    pub fn parse_str(s: &str) -> Result<ShortAddress, ParseError> {
        let mut offset = 0; // Offset into the u8 vector
        let mut hn: bool = false; // Have we seen the high nibble yet?
        let mut short = [0_u8; 2];

        match s.len() {
            5 | 6 => {} // The formats are 4 characters with 2(hex), 1(.-:) delims
            _ => return Err(ParseError::InvalidLength(s.len())),
        }

        for (idx, c) in s.chars().enumerate() {
            if offset >= 2 {
                // We shouln't still be parsing
                return Err(ParseError::InvalidLength(s.len()));
            }

            match c {
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    match hn {
                        false => {
                            // We will match '0' and run this even if the format is 0x
                            hn = true; // Parsed the high nibble
                            short[offset] = (c.to_digit(16).unwrap() as u8) << 4;
                        }
                        true => {
                            hn = false; // Parsed the low nibble
                            short[offset] += c.to_digit(16).unwrap() as u8;
                            offset += 1;
                        }
                    }
                }
                '-' | ':' | '.' => {}
                'x' | 'X' => {
                    match idx {
                        1 => {
                            // If idx = 1, we are possibly parsing 0x1234 format
                            // Reset the offset to zero to ignore the first two characters
                            offset = 0;
                            hn = false;
                        }
                        _ => return Err(ParseError::InvalidCharacter(c, idx)),
                    }
                }
                _ => return Err(ParseError::InvalidCharacter(c, idx)),
            }
        }

        if offset == 2 {
            // A correctly parsed value is exactly 2 u8s
            Ok(ShortAddress { addr: short })
        } else {
            Err(ParseError::InvalidLength(s.len())) // Something slipped through
        }
    }
 }

impl FromStr for ShortAddress {
    type Err = ParseError;
    fn from_str(us: &str) -> Result<ShortAddress, ParseError> {
        ShortAddress::parse_str(us)
    }
}

impl Default for ShortAddress {
    fn default() -> ShortAddress {
        ShortAddress::nil()
    }
}

impl fmt::Debug for ShortAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ShortAddress(\"{}\")",
            self.to_string()
        )
    }
}

impl fmt::Display for ShortAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidLength(found) => write!(
                f,
                "Invalid length; expecting 5 or 6 chars, found {}",
                found
            ),
            ParseError::InvalidCharacter(found, pos) => {
                write!(f, "Invalid character; found `{}` at offset {}", found, pos)
            }
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "ShortAddress parse error"
    }
}

impl Serialize for ShortAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ShortAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ShortAddressVisitor;
        impl<'de> de::Visitor<'de> for ShortAddressVisitor {
            type Value = ShortAddress;

            fn visit_str<E: de::Error>(self, value: &str) -> Result<ShortAddress, E> {
                value.parse().map_err(|err| E::custom(&format!("{}", err)))
            }

            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<ShortAddress, E> {
                ShortAddress::from_bytes(value).map_err(|_| E::invalid_length(value.len(), &self))
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "either a string representation of a MAC address or 6-element byte array"
                )
            }
        }
        deserializer.deserialize_str(ShortAddressVisitor)
    }
}
