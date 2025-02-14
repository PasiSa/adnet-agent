use std::{
    collections::HashMap,
    error::Error,
    net::{SocketAddr, UdpSocket},
};

use rand::prelude::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;

use crate::{
    client::AdNetError,
    Client,
    tasks::parse_commandstr,
};


pub fn start(client: &mut Client, buf: &[u8]) -> Result<bool, Box<dyn Error>> {
    let commandstr = parse_commandstr(buf)?;
    debug!("commandstr vector: {:?}", commandstr);
    if commandstr.len() < 2 {
        return Err(Box::new(AdNetError::new_str("Invalid command string")));
    }
    info!("Handling TASK-003. Secret code: {}", commandstr[1]);

    let mut rng: Pcg64 = Seeder::from(commandstr[1].trim_end()).make_rng();
    let total_length: u32 = rng.gen();
    let total_length = total_length % 40000 + 200000;
    let character: u8 = rng.gen_range(48..=122);
    let bytevector: Vec<u8> = (0..97).map(|_| rng.gen()).collect();

    debug!("Starting transfer of {} bytes repeating character '{}'",
        total_length, character);
    client.write_socket(format!("{} {}",
        total_length, char::from_u32(character as u32).unwrap()).as_bytes())?;

    transmit_loop(total_length, character, &bytevector)?;

    Ok(true)
}


fn transmit_loop(total_length: u32, character: u8, bytevector: &[u8]) ->  Result<(), AdNetError> {
    // TODO: error processing
    let socket = UdpSocket::bind("0.0.0.0:20000").unwrap();
    let mut cumulative = 0;
    let mut received: usize = 0;

    // Hash of sequence numbers and with packet lengths received out of order
    let mut ofo_set: HashMap<u32, u16> = HashMap::new();
    while received < total_length as usize {
        let mut buf = [0; 1500];
        let (recv_len, address) = socket.recv_from(&mut buf).unwrap();
        if (recv_len) < 6 {
            return Err(AdNetError::new_str("Datagram must be at least 6 bytes long."));
        }
        
        // Check that length is correct.
        let lenbytes: [u8; 2] = buf[4..6].try_into().unwrap();
        let len = u16::from_be_bytes(lenbytes);
        if len > 1200 {
            return Err(AdNetError::new(
                format!(
                    "Payload length is {} bytes, but it must not be larger than 1200 bytes.", len)));
        }
        if len as u32 > total_length - received as u32 {
            return Err(AdNetError::new(
                format!(
                    "Receiving a total of {} bytes, but {} was requested.",
                    received + len as usize, total_length)));
        }
        let ulen = len as usize + 6;
        if recv_len != ulen {
            return Err(AdNetError::new(
                format!("Datagram size is {}, but expected {}", recv_len, ulen)));
        }

        if len > 6 {
            if buf[6] != character {
                return Err(AdNetError::new(
                    format!("Datagram payload contains incorrect value: {}", buf[6])));
            }
        }

        let seqbytes: [u8; 4] = buf[0..4].try_into().unwrap();
        let sequence = u32::from_be_bytes(seqbytes);
        if sequence == cumulative + 1 {
            received += len as usize;
            cumulative += 1;

            // check if out-of-order sequence set can be cleaned up with this datagram
            if let Some(ofolen) = ofo_set.get(&cumulative) {
                received += *ofolen as usize;
                ofo_set.remove(&cumulative);
                cumulative += 1;
            }
            send_ack(&socket, address, sequence, bytevector[received % 97]);
        } else {
            if sequence > cumulative {
                ofo_set.insert(sequence, len);
            }
            send_ack(&socket, address, cumulative, bytevector[received % 97]);
        }
    }
    Ok(())
}


fn send_ack(socket: &UdpSocket, address: SocketAddr, sequence: u32, checknum: u8) {
    let mut ack_bytes = Vec::from(sequence.to_be_bytes());
    ack_bytes.push(checknum);  // pseudorandom (but deterministic) check value
    socket.send_to(&ack_bytes, address).unwrap();  // TODO: handle properly
}
