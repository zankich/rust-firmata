extern crate serial;

pub const ENCODER_DATA:             u8 = 0x61; 
pub const ANALOG_MAPPING_QUERY:     u8 = 0x69; 
pub const ANALOG_MAPPING_RESPONSE:  u8 = 0x6A; 
pub const CAPABILITY_QUERY:         u8 = 0x6B;
pub const CAPABILITY_RESPONSE:      u8 = 0x6C; 
pub const PIN_STATE_QUERY:          u8 = 0x6D; 
pub const PIN_STATE_RESPONSE:       u8 = 0x6E; 
pub const EXTENDED_ANALOG:          u8 = 0x6F; 
pub const SERVO_CONFIG:             u8 = 0x70; 
pub const STRING_DATA:              u8 = 0x71; 
pub const STEPPER_DATA:             u8 = 0x72; 
pub const ONEWIRE_DATA:             u8 = 0x73; 
pub const SHIFT_DATA:               u8 = 0x75;
pub const I2C_REQUEST:              u8 = 0x76; 
pub const I2C_REPLY:                u8 = 0x77; 
pub const I2C_CONFIG:               u8 = 0x78; 
pub const REPORT_FIRMWARE:          u8 = 0x79; 
pub const PROTOCOL_VERSION:         u8 = 0xF9; 
pub const SAMPLEING_INTERVAL:       u8 = 0x7A; 
pub const SCHEDULER_DATA:           u8 = 0x7B; 
pub const SYSEX_NON_REALTIME:       u8 = 0x7E; 
pub const SYSEX_REALTIME:           u8 = 0x7F; 
pub const START_SYSEX:              u8 = 0xF0;
pub const END_SYSEX:                u8 = 0xF7;
pub const PIN_MODE:                 u8 = 0xF4;
pub const REPORT_DIGITAL:           u8 = 0xD0;
pub const REPORT_ANALOG:            u8 = 0xC0;
pub const DIGITAL_MESSAGE:          u8 = 0x90;
pub const ANALOG_MESSAGE:           u8 = 0xE0;

pub const INPUT:                    u8 = 0;
pub const OUTPUT:                   u8 = 1;
pub const ANALOG:                   u8 = 2;
pub const PWM:                      u8 = 3;
pub const SERVO:                    u8 = 4;
pub const I2C:                      u8 = 6;
pub const ONEWIRE:                  u8 = 7;
pub const STEPPER:                  u8 = 8;
pub const ENCODER:                  u8 = 9;

use std::str;
use std::io;
use std::io::Read;
use std::path::Path;
use std::thread;
use serial::*;


fn write<T: SerialPort>(port: &mut T, buf: &mut [u8]) -> io::Result<(usize)> {
    return port.write(buf);
}

fn read<T: SerialPort>(port: &mut T, len: i32) -> io::Result<(Vec<u8>)> {
    use std::io::ErrorKind;
    let mut vec: Vec<u8> = vec![];
    let mut len = len;

    loop {
        let buf: &mut [u8; 1] = &mut [0u8];

        match port.read(buf) {
            Ok(_) => {
                vec.push(buf[0]);
                len = len - 1;
                if len == 0 {
                   break; 
                }
            }
            Err(e) => {
                 if e.kind() == ErrorKind::TimedOut {
                    thread::sleep_ms(1);
                    continue
                }
            }
        }
    }

    return Ok(vec);
}

#[derive(Debug)]
pub struct Mode {
    pub mode: u8,
    pub resolution: u8
}

#[derive(Debug)]
pub struct Pin {
   pub  modes: Vec<Mode>,
   pub  analog: bool,
   pub  value: i32,
}

pub struct Board {
    sp: Box<posix::TTYPort>,
    pub pins: Vec<Pin>,
    pub protocol_version: String,
    pub firmware_name: String,
    pub firmware_version: String,
}

impl Board {

    pub fn new(port: &str) -> Self {
        let mut sp = Box::new(posix::TTYPort::open(&Path::new(port)).unwrap());

        sp.reconfigure(&|settings| {
            settings.set_baud_rate(Baud57600).unwrap();
            settings.set_char_size(Bits8);
            settings.set_parity(ParityNone);
            settings.set_stop_bits(Stop1);
            settings.set_flow_control(FlowNone);
            Ok(())
        }).unwrap();

        
        let mut b = Board {
            sp: sp,
            firmware_name: String::new(),
            firmware_version: String::new(),
            protocol_version: String::new(),
            pins: vec![]
        };

        b.query_firmware();
        b.decode();
        b.decode();
        b.query_capabilities();
        b.decode();
        b.query_analog_mapping();
        b.decode();

        return b;
    }

    pub fn query_analog_mapping(&mut self) {
        write(&mut *self.sp, &mut [START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX]).unwrap();
    }

    pub fn query_capabilities(&mut self) {
        write(&mut *self.sp, &mut [START_SYSEX, CAPABILITY_QUERY, END_SYSEX]).unwrap();
    }

    pub fn query_firmware(&mut self) {
        write(&mut *self.sp, &mut [START_SYSEX, REPORT_FIRMWARE, END_SYSEX]).unwrap();
    }

    pub fn report_analog(&mut self, pin: i32, state: i32) {
        write(&mut *self.sp,
            &mut [
                REPORT_ANALOG | pin as u8,
                state as u8
            ]
        ).unwrap();
    }

    pub fn analog_write(&mut self, pin: i32, level: i32) {
        self.pins[pin as usize].value = level;

        write(&mut *self.sp,
            &mut [
                ANALOG_MESSAGE | pin as u8,
                (level & 0x7f) as u8,
                ((level >> 7) & 0x7f) as u8
            ]
        ).unwrap();
    }

    pub fn digital_write(&mut self, pin: i32, level: i32) {
        let port = (pin as f64 / 8f64).floor() as usize;
        let mut value = 0i32;
        let mut i = 0;

        self.pins[pin as usize].value = level;

        while i < 8 {
            if self.pins[8*port+i].value != 0 {
                value = value | (1 << i)
            }
            i += 1;
        }

        write(&mut *self.sp,
            &mut [
                DIGITAL_MESSAGE | port as u8,
                (value & 0x7f) as u8,
                ((value >> 7) & 0x7f) as u8
            ]
        ).unwrap();
    }

    pub fn set_pin_mode(&mut self, pin: i32, mode: u8) {
        write(&mut *self.sp, &mut [PIN_MODE, pin as u8, mode as u8]).unwrap();
    }

    pub fn decode(&mut self) {
        let mut buf = read(&mut *self.sp, 3).unwrap();
        match buf[0] {
            PROTOCOL_VERSION => {
                self.protocol_version = format!("{:o}.{:o}", buf[1], buf[2]);
            },
            ANALOG_MESSAGE...0xEF => {
                let value = buf[1] as i32 | ((buf[2] as i32) << 7);
                let pin = (buf[0] & 0x0F) as usize;
                self.pins[pin+14usize as usize].value = value;
            },
            START_SYSEX => {
                loop {
                    let message = read(&mut *self.sp, 1).unwrap();
                    buf.push(message[0]);
                    if message[0] == END_SYSEX {
                        break;
                    }
                }
                match buf[1] {
                    ANALOG_MAPPING_RESPONSE => {
                        if self.pins.len() > 0 {
                           let mut i = 2;
                           while i < buf.len()-1 {
                               if buf[i] != 127u8 {
                                   self.pins[i-2].analog = true;
                               }
                               i += 1;
                           }
                        }
                    },
                    CAPABILITY_RESPONSE => {
                        let mut pin = 0;
                        let mut i = 2;
                        self.pins = vec![];
                        self.pins.push(Pin{
                            modes: vec![], 
                            analog: false,
                            value: 0,
                        });
                        while i < buf.len()-1 {
                            if buf[i] == 127u8 {
                                pin += 1;
                                i += 1; 
                                self.pins.push(Pin{
                                    modes: vec![], 
                                    analog: false,
                                    value: 0
                                });
                                continue;
                            }
                            self.pins[pin].modes.push(Mode { 
                                mode: buf[i], 
                                resolution: buf[i+1]
                            });
                            i += 2;
                        }
                    },
                    REPORT_FIRMWARE => {
                        self.firmware_version = format!("{:o}.{:o}", buf[2], buf[3]);
                        self.firmware_name = str::from_utf8(&buf[4..buf.len()-1]).unwrap().to_string();
                    },
                    _ => println!("unknown sysex code"),
                }
            },
            _ => println!("bad byte"),
        }
    }
}
