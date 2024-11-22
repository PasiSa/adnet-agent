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

/// Our custom error type for this application.
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


/// State related to one connected client.
pub struct Client {
    stream: TcpStream,
    written: usize,  // how many bytes written to socket
    writestr: String,  // String to be written
    finished: bool,  // True if connection can be terminated (unless write buffer has data)
}

impl Client {
    /// Create client state after connection has been accepted.
    /// The Client object owns the TCP stream, and sets the mio token for it.
    pub fn new(stream: TcpStream, poll: &mut Poll, token: Token) -> Client {
        let mut c = Client {
            stream,
            written: 0,
            writestr: String::new(),
            finished: false,
        };
        poll.registry().register(&mut c.stream, token, Interest::READABLE).unwrap();
        c
    }


    pub fn handle_read_event(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = [0; 1024];
        match self.stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    debug!("Client closed connection");
                    self.finished = true;
                    return Ok(());
                }
                println!("Read {} bytes from client.", n);
                match self.process_command_msg(&buf[..n]) {
                    Ok(finished) => {
                        self.finished = finished;
                        Ok(())
                    },
                    Err(e) => {
                        error!("Error: {}", e);
                        Ok(())
                    },
                }
            },
            Err(e) => Err(Box::new(
                AdNetError::new(format!("Failed to read from client: {}", e))
            ))
        }
    }


    pub fn handle_write_event(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: handle error on write
        let n = self.stream.write(&self.writestr.as_bytes()[self.written..]).unwrap();
        debug!("Wrote {} bytes", n);
        self.written += n;
        Ok(())
    }


    /// Check if we have something to write to the socket, and if so register to
    /// be interested in WRITABLE MIO events.
    /// Returns `true` if we have something to write.
    pub fn check_write_pending(&mut self, poll: &mut Poll, token: Token) -> bool {
        if self.writestr.len() > self.written {
            poll.registry().reregister(
                &mut self.stream,
                token,
                Interest::READABLE | Interest::WRITABLE,
            ).unwrap();
            true
        } else {
            poll.registry().reregister(
                &mut self.stream,
                token,
                Interest::READABLE,
            ).unwrap();
            false
        }
    }


    pub fn is_finished(&self) -> bool {
        self.finished
    }


    fn process_command_msg(&mut self, buf: &[u8]) -> Result<bool, Box<dyn Error>> {
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


    // Return true, if connection is done and can be closed
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

        self.writestr = rng
            .sample_iter(&Alphanumeric)
            .take(len.try_into().unwrap())
            .map(char::from)
            .collect();
        self.written = 0;

        Ok(true)
    }

}
