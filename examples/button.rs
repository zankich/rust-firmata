extern crate firmata;

use firmata::*;
use std::thread;

fn main() {
    let mut b = firmata::Board::new("/dev/ttyACM0");

    println!("firmware version {}", b.firmware_version);
    println!("firmware name {}", b.firmware_name);
    println!("protocol version {}", b.protocol_version);

    let led = 13;
    let button = 2;

    b.set_pin_mode(led, firmata::OUTPUT);
    b.set_pin_mode(button, firmata::INPUT);

    b.report_digital(button, 1);

    loop {
        b.decode();
        if b.pins[button as usize].value == 0 {
            println!("off");
            b.digital_write(led, 0);
        } else {
            println!("on");
            b.digital_write(led, 1);
        }

        thread::sleep_ms(100);
    }
}
