extern crate firmata;

use firmata::*;
use std::sync::{Arc, Mutex};
use std::thread;

fn init(board: Arc<Mutex<firmata::Board>>) {
    {
        let mut b = board.lock().unwrap();
        b.i2c_config(0);
        b.i2c_write(0x09, "o".as_bytes());
        thread::sleep_ms(10);
    }

    let b = board.clone();
    thread::spawn(move || {
        loop {
            b.lock().unwrap().decode();
            b.lock().unwrap().query_firmware();
            thread::sleep_ms(10);
        }
    });
}

fn set_rgb(board: Arc<Mutex<firmata::Board>>, rgb: [u8; 3]) {
    let mut b = board.lock().unwrap();
    b.i2c_write(0x09, "n".as_bytes());
    b.i2c_write(0x09, &rgb);
}

fn read_rgb(board: Arc<Mutex<firmata::Board>>) -> Vec<u8> {
    {
        let mut b = board.lock().unwrap();
        b.i2c_write(0x09, "g".as_bytes());
        b.i2c_read(0x09, 3);
    }
    loop {
        {
            let mut b = board.lock().unwrap();
            if b.i2c_data.iter().count() > 0 {
                return b.i2c_data.pop().unwrap().data;
            }
        }
        thread::sleep_ms(10);
    }
}

fn main() {
    let board = Arc::new(Mutex::new(firmata::Board::new("/dev/ttyACM0")));

    init(board.clone());

    set_rgb(board.clone(), [255, 0, 0]);
    println!("rgb: {:?}", read_rgb(board.clone()));
    thread::sleep_ms(1000);

    set_rgb(board.clone(), [0, 255, 0]);
    println!("rgb: {:?}", read_rgb(board.clone()));
    thread::sleep_ms(1000);

    set_rgb(board.clone(), [0, 0, 255]);
    println!("rgb: {:?}", read_rgb(board.clone()));
    thread::sleep_ms(1000);
}
