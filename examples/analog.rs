extern crate firmata;

use firmata::*;
use std::thread;

fn main() {
    let mut b = firmata::Board::new("/dev/ttyACM0");

    let pin = 14; // A0

    println!("firmware version {}", b.firmware_version);
    println!("firmware name {}", b.firmware_name);
    println!("protocol version {}", b.protocol_version);

    b.set_pin_mode(pin, firmata::ANALOG);

    b.report_analog(pin, 1);

    loop {
        b.decode();
        println!("analog value: {:o}", b.pins[pin as usize].value);
        thread::sleep_ms(10);
    }
}
