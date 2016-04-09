//! This module contains a client implementation of the
//! [Firmata Protocol](https://github.com/firmata/protocol)
use std::str;
use std::io;
use std::thread;
use std::time::Duration;
use std::io::{Write, Read, Error, Result, ErrorKind};

pub const ENCODER_DATA: u8 = 0x61;
pub const ANALOG_MAPPING_QUERY: u8 = 0x69;
pub const ANALOG_MAPPING_RESPONSE: u8 = 0x6A;
pub const CAPABILITY_QUERY: u8 = 0x6B;
pub const CAPABILITY_RESPONSE: u8 = 0x6C;
pub const PIN_STATE_QUERY: u8 = 0x6D;
pub const PIN_STATE_RESPONSE: u8 = 0x6E;
pub const EXTENDED_ANALOG: u8 = 0x6F;
pub const SERVO_CONFIG: u8 = 0x70;
pub const STRING_DATA: u8 = 0x71;
pub const STEPPER_DATA: u8 = 0x72;
pub const ONEWIRE_DATA: u8 = 0x73;
pub const SHIFT_DATA: u8 = 0x75;
pub const I2C_REQUEST: u8 = 0x76;
pub const I2C_REPLY: u8 = 0x77;
pub const I2C_CONFIG: u8 = 0x78;
pub const I2C_MODE_WRITE: u8 = 0x00;
pub const I2C_MODE_READ: u8 = 0x01;
pub const REPORT_FIRMWARE: u8 = 0x79;
pub const PROTOCOL_VERSION: u8 = 0xF9;
pub const SAMPLEING_INTERVAL: u8 = 0x7A;
pub const SCHEDULER_DATA: u8 = 0x7B;
pub const SYSEX_NON_REALTIME: u8 = 0x7E;
pub const SYSEX_REALTIME: u8 = 0x7F;
pub const START_SYSEX: u8 = 0xF0;
pub const END_SYSEX: u8 = 0xF7;
pub const PIN_MODE: u8 = 0xF4;
pub const REPORT_DIGITAL: u8 = 0xD0;
pub const REPORT_ANALOG: u8 = 0xC0;
pub const DIGITAL_MESSAGE: u8 = 0x90;
pub const ANALOG_MESSAGE: u8 = 0xE0;

pub const INPUT: u8 = 0;
pub const OUTPUT: u8 = 1;
pub const ANALOG: u8 = 2;
pub const PWM: u8 = 3;
pub const SERVO: u8 = 4;
pub const I2C: u8 = 6;
pub const ONEWIRE: u8 = 7;
pub const STEPPER: u8 = 8;
pub const ENCODER: u8 = 9;

fn read<T: io::Read>(port: &mut T, len: i32) -> Result<(Vec<u8>)> {
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
                    thread::sleep(Duration::from_millis(1));
                    continue
                }
            } }
    }

    return Ok(vec);
}

/// A structure representing an I2C reply.
#[derive(Debug)]
pub struct I2CReply {
    pub address: i32,
    pub register: i32,
    pub data: Vec<u8>,
}

/// A structure representing an available pin mode.
#[derive(Debug)]
pub struct Mode {
    pub mode: u8,
    pub resolution: u8
}

/// A structure representing the current state and configuration of a pin.
#[derive(Debug)]
pub struct Pin {
    pub modes: Vec<Mode>,
    pub analog: bool,
    pub value: i32,
    pub mode: u8,
}

/// A trait for implementing firmata boards.
pub trait Firmata {
    /// This function returns the raw I2C replies that have been read from
    /// the board.
    fn i2c_data(&mut self) -> &mut Vec<I2CReply>;
    /// This function returns the pins that the board has access to.
    fn pins(&mut self) -> &Vec<Pin>;
    /// This function returns the current firmata protocol version.
    fn protocol_version(&mut self) -> &String;
    /// This function returns the firmware name.
    fn firmware_name(&mut self) -> &String;
    /// This function returns the firmware version.
    fn firmware_version(&mut self) -> &String;
    /// This function queries the board for available analog pins.
    fn query_analog_mapping(&mut self) -> Result<()>;
    /// This function queries the board for all available capabilities.
    fn query_capabilities(&mut self) -> Result<()>;
    /// This function queries the board for current firmware and protocol
    /// information.
    fn query_firmware(&mut self) -> Result<()>;
    /// This function configures the `delay` in microseconds for I2C devices
    /// that require a delay between when the register is written to and the
    /// data in that register can be read.
    fn i2c_config(&mut self, delay: i32) -> Result<()>;
    /// This function reads `size` bytes from I2C device at the specified
    /// `address`.
    fn i2c_read(&mut self, address: i32, size: i32) -> Result<()>;
    /// This function writes `data` to the I2C device at
    /// the specified `address`.
    fn i2c_write(&mut self, address: i32, data: &[u8]) -> Result<()>;
    /// This function sets the digital reporting `state`
    /// of the specified `pin`.
    fn report_digital(&mut self, pin: i32, state: i32) -> Result<()>;
    /// This function sets the analog reporting `state` of the specified `pin`.
    fn report_analog(&mut self, pin: i32, state: i32) -> Result<()>;
    /// This function writes `level` to the analog `pin`.
    fn analog_write(&mut self, pin: i32, level: i32) -> Result<()>;
    /// This function writes `level` to the digital `pin`.
    fn digital_write(&mut self, pin: i32, level: i32) -> Result<()>;
    /// This function sets the `mode` of the specified `pin`.
    fn set_pin_mode(&mut self, pin: i32, mode: u8) -> Result<()>;
    /// This function reads from the firmata device and parses one firmata
    /// message.
    fn read_and_decode(&mut self) -> Result<()>;
}

/// A structure representing a firmata board.
pub struct Board<T: io::Read+io::Write> {
    pub connection: Box<T>,
    pub pins: Vec<Pin>,
    pub i2c_data: Vec<I2CReply>,
    pub protocol_version: String,
    pub firmware_name: String,
    pub firmware_version: String,
}

impl<T: io::Read+io::Write> Board<T> {
    /// Creates a new `Board` given an `io::Read+io::Write`.
    pub fn new(connection: Box<T>) -> Result<Board<T>> {
        let mut b = Board {
            connection: connection,
            firmware_name: String::new(),
            firmware_version: String::new(),
            protocol_version: String::new(),
            pins: vec![],
            i2c_data: vec![],
        };

        try!(b.query_firmware());
        try!(b.read_and_decode());
        try!(b.read_and_decode());
        try!(b.query_capabilities());
        try!(b.read_and_decode());
        try!(b.query_analog_mapping());
        try!(b.read_and_decode());
        try!(b.report_digital(0, 1));
        try!(b.report_digital(1, 1));

        return Ok(b);
    }
}

impl<T:io::Read+io::Write> Firmata for Board<T> {
    fn pins(&mut self) -> &Vec<Pin> {
        &self.pins
    }
    fn protocol_version(&mut self) -> &String {
        &self.protocol_version
    }
    fn firmware_name(&mut self) -> &String {
        &self.firmware_name
    }
    fn firmware_version(&mut self) -> &String {
        &self.firmware_version
    }
    fn i2c_data(&mut self) -> &mut Vec<I2CReply> {
        &mut self.i2c_data
    }
    fn query_analog_mapping(&mut self) -> Result<()> {
        self.connection
            .write(&mut [START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX])
            .map(|_|())
    }

    fn query_capabilities(&mut self) -> Result<()> {
        self.connection
            .write(&mut [START_SYSEX, CAPABILITY_QUERY, END_SYSEX])
            .map(|_|())
    }

    fn query_firmware(&mut self) -> Result<()> {
        self.connection
            .write(&mut [START_SYSEX, REPORT_FIRMWARE, END_SYSEX])
            .map(|_|())
    }

    fn i2c_config(&mut self, delay: i32) -> Result<()> {
        self.connection.write(
            &mut [
                START_SYSEX,
                I2C_CONFIG,
                (delay & 0xFF) as u8,
                (delay >> 8 & 0xFF) as u8,
                END_SYSEX
            ]
        ).map(|_|())
    }

    fn i2c_read(&mut self, address: i32, size: i32) -> Result<()> {
        self.connection.write(
            &mut [
                START_SYSEX,
                I2C_REQUEST,
                address as u8,
                (I2C_MODE_READ << 3),
                ((size as u8) & 0x7F),
                (((size) >> 7) & 0x7F) as u8,
                END_SYSEX
            ]
        ).map(|_|())
    }

    fn i2c_write(&mut self, address: i32, data: &[u8]) -> Result<()> {
        let mut buf = vec![];

        buf.push(START_SYSEX);
        buf.push(I2C_REQUEST);
        buf.push(address as u8);
        buf.push(I2C_MODE_WRITE << 3);

        for i in data.iter() {
            buf.push(i & 0x7F);
            buf.push((((*i as i32) >> 7) & 0x7F) as u8);
        }

        buf.push(END_SYSEX);

        self.connection.write(&mut buf[..]).map(|_|())
    }

    fn report_digital(&mut self, pin: i32, state: i32) -> Result<()> {
        self.connection.write(
            &mut [
                REPORT_DIGITAL | pin as u8,
                state as u8
            ]
        ).map(|_|())
    }

    fn report_analog(&mut self, pin: i32, state: i32) -> Result<()> {
        self.connection.write(
            &mut [
                REPORT_ANALOG | pin as u8,
                state as u8
            ]
        ).map(|_|())
    }

    fn analog_write(&mut self, pin: i32, level: i32) -> Result<()> {
        self.pins[pin as usize].value = level;

        self.connection.write(
            &mut [
                ANALOG_MESSAGE | pin as u8,
                (level & 0x7f) as u8,
                ((level >> 7) & 0x7f) as u8
            ]
        ).map(|_|())
    }

    fn digital_write(&mut self, pin: i32, level: i32) -> Result<()> {
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

        self.connection.write(
            &mut [
                DIGITAL_MESSAGE | port as u8,
                (value & 0x7f) as u8,
                ((value >> 7) & 0x7f) as u8
            ]
        ).map(|_|())
    }

    fn set_pin_mode(&mut self, pin: i32, mode: u8) -> Result<()> {
        self.pins[pin as usize].mode = mode;
        self.connection.write(&mut [PIN_MODE, pin as u8, mode as u8]).map(|_|())
    }

    fn read_and_decode(&mut self) -> Result<()> {
        let mut buf = try!(read(&mut self.connection, 3));
        match buf[0] {
            PROTOCOL_VERSION => {
                self.protocol_version = format!("{:o}.{:o}", buf[1], buf[2]);
                Ok(())
            },
            ANALOG_MESSAGE...0xEF => {
                let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
                let pin = ((buf[0] as i32) & 0x0F) + 14;

                if self.pins.len() as i32 > pin {
                    self.pins[pin as usize].value = value;
                }
                Ok(())
            },
            DIGITAL_MESSAGE...0x9F => {
                let port = (buf[0] as i32) & 0x0F;
                let value = (buf[1] as i32) | ((buf[2] as i32) << 7);

                for i in 0..8 {
                    let pin = (8 * port) + i;

                    if self.pins.len() as i32 > pin {
                        if self.pins[pin as usize].mode == INPUT {
                            self.pins[pin as usize].value = (value >> (i & 0x07)) & 0x01;
                        }
                    }
                }
                Ok(())
            },
            START_SYSEX => {
                loop {
                    let message = try!(read(&mut *self.connection, 1));
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
                        Ok(())
                    },
                    CAPABILITY_RESPONSE => {
                        let mut pin = 0;
                        let mut i = 2;
                        self.pins = vec![];
                        self.pins.push(Pin{
                            modes: vec![],
                            analog: false,
                            value: 0,
                            mode: 0,
                        });
                        while i < buf.len()-1 {
                            if buf[i] == 127u8 {
                                pin += 1;
                                i += 1;
                                self.pins.push(Pin{
                                    modes: vec![],
                                    analog: false,
                                    value: 0,
                                    mode: 0,
                                });
                                continue;
                            }
                            self.pins[pin].modes.push(Mode {
                                mode: buf[i],
                                resolution: buf[i+1]
                            });
                            i += 2;
                        }
                        Ok(())
                    },
                    REPORT_FIRMWARE => {
                        self.firmware_version = format!("{:o}.{:o}", buf[2], buf[3]);
                        self.firmware_name = str::from_utf8(&buf[4..buf.len()-1]).unwrap().to_string();
                        Ok(())
                    },
                    I2C_REPLY => {
                        let len = buf.len();
                        let mut reply = I2CReply{
                            address:  (buf[2] as i32) | ((buf[3] as i32) << 7),
                            register: (buf[4] as i32) | ((buf[5] as i32) << 7),
                            data:     vec![buf[6] | buf[7]<<7],
                        };
                        let mut i = 8;

                        while i < len-1 {
                            if buf[i] == 0xF7 {
                                break
                            }
                            if i+2 > len {
                                break
                            }
                            reply.data.push(buf[i] | buf[i+1] << 7);
                            i += 2;
                        }
                        self.i2c_data.push(reply);
                        Ok(())
                    },
                    _ => Err(Error::new(ErrorKind::Other, "unknown sysex code")),
                }
            },
            _ => Err(Error::new(ErrorKind::Other,"bad byte")),
        }
    }

}
