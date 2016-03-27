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

    let led = 13;
    let button = 2;

    b.set_pin_mode(led, firmata::OUTPUT);
    b.set_pin_mode(button, firmata::INPUT);

    b.report_digital(button, 1);

    loop {
        b.read_and_decode();
        if b.pins()[button as usize].value == 0 {
            println!("off");
            b.digital_write(led, 0);
        } else {
            println!("on");
            b.digital_write(led, 1);
        }

        thread::sleep_ms(100);
    }
}
