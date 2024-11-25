use std::error::Error;

use rand::{distributions::Alphanumeric, prelude::*};
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
    info!("Handling TASK-002. Secret code: {}", commandstr[1]);

    let mut rng: Pcg64 = Seeder::from(commandstr[1].trim_end()).make_rng();
    let len: u32 = rng.gen();
    let len: u32 = len % 20000 + 90000;
    debug!("Length is {}", len);

    client.write_string(rng
        .sample_iter(&Alphanumeric)
        .take(len.try_into().unwrap())
        .map(char::from)
        .collect()
    );

    Ok(true)
}
