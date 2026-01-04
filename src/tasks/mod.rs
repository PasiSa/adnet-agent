use std::error::Error;

use crate::client::AdNetError;


pub fn parse_commandstr(buf: &[u8]) -> Result<Vec<String>, Box<dyn Error>> {
    match String::from_utf8(buf.to_vec()) {
        Ok(s) => {
            let v: Vec<String> = s.split_whitespace()
                .map(|s| s.to_string())
                .collect();
            return Ok(v);
        },
        Err(_) => {
            return Err(Box::new(AdNetError::new_str("Could not parse command message")));
        }
    }
}

pub mod cli;
pub mod srv;
pub mod udp;
