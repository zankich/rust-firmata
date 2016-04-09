extern crate firmata;
extern crate serial;

use firmata::*;
use serial::*;
use std::sync::{Arc, Mutex};
use std::thread;

fn init<T: firmata::Firmata>(board: &Arc<Mutex<T>>) {
    let mut b = board.lock().unwrap();
    b.i2c_config(0);
    b.i2c_write(0x09, "o".as_bytes());
    thread::sleep_ms(10);
}

fn set_rgb<T: firmata::Firmata>(board: &Arc<Mutex<T>>, rgb: [u8; 3]) {
    let mut b = board.lock().unwrap();
    b.i2c_write(0x09, "n".as_bytes());
    b.i2c_write(0x09, &rgb);
}

fn read_rgb<T: firmata::Firmata>(board: &Arc<Mutex<T>>) -> Vec<u8> {
    {
        let mut b = board.lock().unwrap();
        b.i2c_write(0x09, "g".as_bytes());
        b.i2c_read(0x09, 3);
    }
    loop {
        {
            let mut b = board.lock().unwrap();
            if b.i2c_data().iter().count() > 0 {
                return b.i2c_data().pop().unwrap().data;
            }
        }
        thread::sleep_ms(10);
    }
}

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

    let board = Arc::new(Mutex::new(firmata::Board::new(Box::new(sp)).unwrap()));

    {
        let b = board.clone();
        thread::spawn(move || {
            loop {
                b.lock().unwrap().read_and_decode();
                b.lock().unwrap().query_firmware();
                thread::sleep_ms(10);
            }
        });
    }

    init(&board);

    set_rgb(&board, [255, 0, 0]);
    println!("rgb: {:?}", read_rgb(&board));
    thread::sleep_ms(1000);

    set_rgb(&board, [0, 255, 0]);
    println!("rgb: {:?}", read_rgb(&board));
    thread::sleep_ms(1000);

    set_rgb(&board, [0, 0, 255]);
    println!("rgb: {:?}", read_rgb(&board));
    thread::sleep_ms(1000);
}
