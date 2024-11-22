#[macro_use]
extern crate log;

use std::{
    collections::HashMap,
    error::Error,
};

use env_logger::Env;
use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;

use crate::{
    args::Args,
    client::Client,
    mio_tokens::TokenManager,
};


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(
        Env::default()
            .default_filter_or("info"))
        .format_timestamp_nanos().init();

    info!("adnet-agent listening for connections");

    // Parse command line arguments.
    let args = Args::new();

    // Set up MIO event engine for handling concurrent I/O operations.
    let mut tokenmanager = TokenManager::new();
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);
 
    // Set up a listening TCP socket and bind it at address given in
    // command line argument. Register a MIO token for it for triggering
    // events when incoming connections arrive.
    let addr = args.listen().parse()?;
    let mut server = TcpListener::bind(addr)?;
    let listen_token = tokenmanager.allocate_token();
    poll.registry()
        .register(&mut server, listen_token, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();

    loop {
        // Wait for next MIO event.
        poll.poll(&mut events, None)?;

        // Process each event.
        for event in events.iter() {

            // Check if there is a new connection arriving in the passive listening socket.
            if event.token() == listen_token {
                let (stream, address) = server.accept().unwrap();

                // Add new MIO token for each new socket and register it.
                debug!("Accepting connection from {}", address.to_string());
                let token = tokenmanager.allocate_token();
                let c = Client::new(stream, &mut poll, token);
                clients.insert(token, c);

                continue;
            }

            // Go through the active sockets, and check there is something to read,
            // or if it is possible to write to socket (i.e., there is room in
            // the socket send buffer).
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

                // If we have done all work with this client, clean it up.
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
