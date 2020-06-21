extern crate libpulse_binding as pulse;

use std::error::Error;

mod connect;
mod watcher;

fn main() -> Result<(), Box<dyn Error>> {
    let mac = get_mac_from_args()?;
    let queue = connect::start();

    watcher::start(&mac, queue)?;

    Ok(())
}

fn get_mac_from_args() -> Result<String, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match (args.next(), args.next()) {
        (Some(mac), None) => {
            if valid_mac(&mac) {
                Ok(mac)
            } else {
                Err("Invalid MAC")?
            }
        }

        _ => Err("Usage: pulseaudio-headphones-connect MAC")?,
    }
}

fn valid_mac(mac: &str) -> bool {
    mac.split(':').all(|x| u8::from_str_radix(x, 16).is_ok()) && mac.split(':').count() == 6
}
