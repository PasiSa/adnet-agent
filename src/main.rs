#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::error::Error;

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

use crate::client::Client;
use crate::mio_tokens::TokenManager;


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format_timestamp_nanos().init();
    println!("adnet-agent listening for connections");

    let mut tokenmanager = TokenManager::new();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
 
    let addr = "127.0.0.1:12345".parse()?;
    let mut server = TcpListener::bind(addr)?;
    let listen_token = tokenmanager.allocate_token();
    poll.registry()
        .register(&mut server, listen_token, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();

    loop {
        // Poll Mio for events, blocking until we get an event.
        poll.poll(&mut events, None)?;

        // Process each event.
        for event in events.iter() {
            if event.token() == listen_token {
                let (stream, address) = server.accept().unwrap();

                debug!("Accepting connection from {}", address.to_string());
                let token = tokenmanager.allocate_token();
                let c = Client::new(stream, &mut poll, token);
                clients.insert(token, c);

                continue;
            }

            if let Some(c) = clients.get_mut(&event.token()) {
                match c.read() {
                    Ok(n) => {
                        if n == 0 {  // closing connection
                            tokenmanager.free_token(event.token());
                            clients.remove(&event.token());
                        }
                    },
                    Err(e) => {
                        error!("Client error: {}", e);
                        tokenmanager.free_token(event.token());
                        clients.remove(&event.token());
                    }
                };
            } else {
                error!("Unknown token!");
            }
        }
    }
}

mod client;
mod mio_tokens;
