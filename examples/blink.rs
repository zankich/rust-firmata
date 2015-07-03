extern crate firmata;

use firmata::*;
use std::thread;

fn main() {
    let mut b = firmata::Board::new("/dev/ttyACM0");

    println!("firmware version {}", b.firmware_version);
    println!("firmware name {}", b.firmware_name);
    println!("protocol version {}", b.protocol_version);

    b.set_pin_mode(13, firmata::OUTPUT);

    let mut i = 0;

    loop {
        thread::sleep_ms(400);
        println!("{}",i);
        b.digital_write(13, i);
        i ^= 1;
    }
}
