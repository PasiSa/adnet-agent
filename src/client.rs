use std::{
    error::Error,
    fmt,
    io::{Read, Write},
};

use mio::{Interest, Poll, Token};

use rand::{distributions::Alphanumeric, prelude::*};
use rand_seeder::Seeder;
use rand_pcg::Pcg64;

use mio::net::TcpStream;

#[derive(Debug)]
struct AdNetError {
    msg: String,
}

impl fmt::Display for AdNetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AdNet error: {}", self.msg)
    }
}

impl Error for AdNetError {}

impl AdNetError {
    pub fn new(msg: String) -> AdNetError {
        AdNetError { msg: msg }
    }
    pub fn new_str(msg: &str) -> AdNetError {
        AdNetError { msg: String::from(msg) }
    }
}

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream, poll: &mut Poll, token: Token) -> Client {
        let mut c = Client { stream };
        poll.registry().register(&mut c.stream, token, Interest::READABLE).unwrap();
        c
    }

    // Returns true if client is done and can be removed
    pub fn read(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut buf = [0; 1024];
        match self.stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    debug!("Client closed connection");
                    return Ok(true);
                }
                println!("Read {} bytes from client.", n);
                match self.handle_client(&buf[..n]) {
                    Ok(done) => Ok(done),
                    Err(e) => {
                        error!("Error: {}", e);
                        Ok(false)
                    },
                }
            },
            Err(e) => Err(Box::new(
                AdNetError::new(format!("Failed to read from client: {}", e))
            ))
        }
    }

    fn handle_client(&mut self, buf: &[u8]) -> Result<bool, Box<dyn Error>> {
        if buf.len() < 8 {
            return Err(Box::new(AdNetError::new_str("Too short command message")));
        }
        let string = String::from_utf8(buf[..8].to_vec())?;
        debug!("Read string: {}", string);
        match string.as_str() {
            "TASK-001" => self.handle_task001(buf),
            _ => {
                error!("Invalid command: {}", string);
                Err(Box::new(AdNetError::new(format!("Invalid command: {}", string))))
            },
        }
    }

    fn handle_task001(&mut self, buf: &[u8]) -> Result<bool, Box<dyn Error>> {
        let codestr = match String::from_utf8(buf[9..].to_vec()) {
            Ok(s) => s,
            Err(_) => {
                return Err(Box::new(AdNetError::new_str("Invalid code")));
            }
        };
        info!("Handling TASK-001. Secret code: {}", codestr.trim_end());

        let mut rng: Pcg64 = Seeder::from(codestr.trim_end()).make_rng();
        let len: u32 = rng.gen();
        let len = len % 20000 + 90000;
        debug!("Length is {}", len);

        let s: String = rng
            .sample_iter(&Alphanumeric)
            .take(len.try_into().unwrap())
            .map(char::from)
            .collect();

        let mut total: usize = 0;
        while total < len.try_into().unwrap() {
            let n = self.stream.write(&s.as_bytes()[total..]).unwrap();
            debug!("Wrote {} bytes", n);
            total += n;
        }

        Ok(true)
    }

}
