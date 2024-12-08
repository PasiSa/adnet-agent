use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
    thread
};

use rand::prelude::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;

use crate::{
    client::AdNetError,
    Client,
    tasks::parse_commandstr,
};


struct RandValues {
    length: u32,
    character: u8,
}


pub fn start(mut _client: &Client, buf: &[u8]) -> Result<bool, Box<dyn Error>> {
    let commandstr = parse_commandstr(buf)?;
    debug!("commandstr vector: {:?}", commandstr);
    if commandstr.len() < 3 {
        return Err(Box::new(AdNetError::new_str("Invalid command string")));
    }
    info!("Handling TASK-002. Secret code: {}", commandstr[1]);
    let addrstr = Arc::new(commandstr[2].clone());

    let mut rng: Pcg64 = Seeder::from(commandstr[1].trim_end()).make_rng();
    let mut handles = vec![];
    for _i in 0..3 {
        let mut values: Vec<RandValues> = Vec::new();
        for _j in 0..3 {
            let len: u32 = rng.gen();
            let char: u8 = rng.gen();
            values.push( RandValues{
                length: len % 40000 + 190000,
                character: char % 20 + 65,
           });
        }
        let arc_clone = Arc::clone(&addrstr);
        let handle = thread::spawn(move || start_client(arc_clone, values));
        handles.push(handle);
        thread::sleep(Duration::from_millis(100));
    }

    for handle in handles {
        handle.join().unwrap().unwrap();
    }

    Ok(true)
}


fn start_client(addrstr: Arc<String>, values: Vec<RandValues>) ->  Result<(), AdNetError> {
    debug!("Connecting {}", addrstr);
    let mut client = TcpStream::connect(&addrstr.to_string());
    if client.is_err() {
        return Err(AdNetError::new(format!("Error connecting socket to {}.", addrstr)));
    }
    let client = client.as_mut().unwrap();
    for transfer in values {
        debug!("Starting transfer of {} bytes repeating character '{}'",
            transfer.length, transfer.character);
        let mut buf = transfer.length.to_be_bytes().to_vec();
        buf.push(transfer.character);
        if client.write_all(&buf).is_err() {
            return Err(AdNetError::new_str("Error writing length."));
        }

        let mut numread: usize = 0;
        let mut buf = [0; 10000];

        while numread < transfer.length.try_into().unwrap() {
            match client.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        return Err(AdNetError::new_str("Connection closed prematurely."));
                    }
                    numread += n;
                },
                Err(e) => {
                    return Err(AdNetError::new(format!("Error reading from socket: {}.", e)));
                }
            };
            if buf[0] != transfer.character {
                return Err(AdNetError::new(format!("Invalid character in transmission: {}", buf[0])));
            }
            thread::sleep(Duration::from_millis(100));
        }

    }
    Ok(())
}
