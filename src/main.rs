#[macro_use]
extern crate log;

use std::{
    collections::HashMap,
    error::Error,
};

use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;

use crate::{
    args::Args,
    client::Client,
    mio_tokens::TokenManager,
};


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format_timestamp_nanos().init();
    println!("adnet-agent listening for connections");

    let args = Args::new();

    let mut tokenmanager = TokenManager::new();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
 
    let addr = args.listen().parse()?;
    let mut server = TcpListener::bind(addr)?;
    let listen_token = tokenmanager.allocate_token();
    poll.registry()
        .register(&mut server, listen_token, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();

    loop {
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
                if event.is_readable() {
                    if let Err(e) = c.handle_read_event() {
                        error!("Client error: {}", e);
                        tokenmanager.free_token(event.token());
                        clients.remove(&event.token());
                        break;
                    }
                }

                if event.is_writable() {
                    if let Err(e) = c.handle_write_event() {
                        error!("Client error: {}", e);
                        tokenmanager.free_token(event.token());
                        clients.remove(&event.token());
                        break;
                    }
                }

                if !c.check_write_pending(&mut poll, event.token()) && c.is_finished() {
                    debug!("Client finishing");
                    tokenmanager.free_token(event.token());
                    clients.remove(&event.token());
                }
            } else {
                error!("Unknown token!");
            }
        }
    }
}

mod args;
mod client;
mod mio_tokens;
