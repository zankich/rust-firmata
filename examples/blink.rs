extern crate firmata;
extern crate serial;

use firmata::*;
use serial::*;
use std::thread;

fn main() {
    let mut sp = serial::open("/dev/ttyACM0").unwrap();

    sp.reconfigure(&|settings| {
        settings.set_baud_rate(Baud57600).unwrap();
        settings.set_char_size(Bits8);
        settings.set_parity(ParityNone);
        settings.set_stop_bits(Stop1);
        settings.set_flow_control(FlowNone);
        Ok(())
    }).unwrap();

    let mut b = firmata::Board::new(Box::new(sp)).unwrap();

    println!("firmware version {}", b.firmware_version());
    println!("firmware name {}", b.firmware_name());
    println!("protocol version {}", b.protocol_version());

    b.set_pin_mode(13, firmata::OUTPUT);

    let mut i = 0;

    loop {
        thread::sleep_ms(400);
        println!("{}",i);
        b.digital_write(13, i);
        i ^= 1;
    }
}
