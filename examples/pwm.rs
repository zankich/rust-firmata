extern crate firmata;

use firmata::*;
use std::thread;

fn main() {
    let mut b = firmata::Board::new("/dev/ttyACM0");

    let pin = 3;

    println!("firmware version {}", b.firmware_version);
    println!("firmware name {}", b.firmware_name);
    println!("protocol version {}", b.protocol_version);

    b.set_pin_mode(pin, firmata::PWM);

    loop {
        for value in 0..255 {
            b.analog_write(pin, value);
            println!("{}", value);
            thread::sleep_ms(10);
        }
    }
}
