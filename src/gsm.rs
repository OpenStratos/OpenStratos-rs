use std::{io, thread};
use std::io::{BufReader, BufRead, Write};
use std::time::Duration;

use wiringpi;
use serial;
use time;
use log::LogLevel::*;

use logger::Logger;

use serial::posix::TTYPort;
use wiringpi::pin::{InputPin, OutputPin, Value};

const GSM_SERIAL: &'static str = "/dev/ttyUSB0";
const GSM_MAX_BAT: f64 = 4.2;
const GSM_MIN_BAT: f64 = 3.7;
const MAIN_MAX_BAT: f64 = 8.4 * 2660f64 / (2660 + 7420) as f64; // Measured Ohms in voltage divider
const MAIN_MIN_BAT: f64 = 7.4 * MAIN_MAX_BAT / 8.4;

pub struct Gsm {
    serial: BufReader<TTYPort>,
    logger: Logger,
    command_logger: Logger,
    power_pin: OutputPin<wiringpi::pin::WiringPi>,
    status_pin: InputPin<wiringpi::pin::WiringPi>,
}

impl Gsm {
    pub fn initialize(wiring_pi: &wiringpi::WiringPi<wiringpi::pin::WiringPi>)
                      -> Result<Gsm, io::Error> {
        Ok(Gsm {
            serial: BufReader::new(try!(serial::open(GSM_SERIAL))),
            logger: try!(Logger::new("data/logs/GSM", "GSM", "GSM")),
            command_logger: try!(Logger::new("data/logs/GSMCommands", "GSMCommands", "GSMCommands")),
            power_pin: wiring_pi.output_pin(7),
            status_pin: wiring_pi.input_pin(21),
        })
    }

    pub fn is_on(&self) -> bool {
        self.status_pin.digital_read() == Value::High
    }

    pub fn has_connectivity(&mut self) -> Result<bool, io::Error> {
        if self.is_on() {
            let response = try!(self.send_command_read("AT+CREG?"));
            Ok(response[1] == "+CREG: 0,1" || response[1] == "+CREG: 0,5")
        } else {
            error!("Trying to check GSM connectivity, but GSM was off!");
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "GSM is off"))
        }
    }

    pub fn turn_on(&mut self) {
        self.logger.log("Turning GSM on…", Info);

        if self.is_on() {
            warn!("Trying to turn GSM on, but GSM was already on!");
            self.logger.log("GSM on.", Info);
        } else {
            self.power_pin.digital_write(Value::Low);
            thread::sleep(Duration::from_secs(2));
            self.power_pin.digital_write(Value::High);

            thread::sleep(Duration::from_secs(3));
            self.logger.log("GSM on.", Info);
        }
    }

    pub fn turn_off(&mut self) {
        self.logger.log("Turning GSM off…", Info);

        if self.is_on() {
            warn!("Trying to turn GSM off, but GSM was already off!");
            self.logger.log("GSM off.", Info);
        } else {
            self.power_pin.digital_write(Value::Low);
            thread::sleep(Duration::from_secs(2));
            self.power_pin.digital_write(Value::High);

            thread::sleep(Duration::from_secs(3));
            self.logger.log("GSM off.", Info);
        }
    }

    pub fn send_sms(&mut self, message: String, number: String) -> Result<(), io::Error> {
        if self.is_on() {
            self.logger.log(&format!("Sending SMS: \"{}\" ({} characters) to number \"{}\"…",
                                     message,
                                     message.len(),
                                     number),
                            Info);
            if message.len() > 160 {
                self.logger.log("Trying to send SMS longer than 160 characters!", Error);
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "message too long"));
            }

            if cfg!(feature = "sms") {
                if try!(self.send_command_read("AT+CMGF=1"))[1] != "OK" {
                    // TODO check bounds
                    self.logger.log("No OK received sending SMS on 'AT+CMGD=1' response.", Error);
                    return Err(io::Error::new(io::ErrorKind::Other,
                                              "no OK received on 'AT+CMGD=1' response"));
                }

                if try!(self.send_command_read(&format!("AT+CMGS=\"{}\"", number)))[1] != "> " {
                    self.logger.log("No prompt received sending SMS on 'AT+CMGS' response.",
                                    Error);
                    return Err(io::Error::new(io::ErrorKind::Other,
                                              "no prompt received on 'AT+CMGS' response"));
                }

                try!(writeln!(self.serial.get_mut(), "{}", message));
                self.command_logger.log(&format!("Sent: '{}'", message), Info);
                let mut response = String::new();
                while try!(self.serial.read_line(&mut response)) != 0 {
                    self.command_logger.log(&format!("Received: '{}'", response), Info)
                }
                try!(writeln!(self.serial.get_mut(), ""));
                try!(self.serial.read_line(&mut response));
                try!(self.serial.get_mut().write_all(&[0x1A]));
                let start = time::precise_time_s();
                while time::precise_time_s() - start < 60f64 {
                    if try!(self.serial.read_line(&mut response)) != 0 {
                        break;
                    }
                }

                // Read +CMGS response
                try!(self.serial.read_line(&mut response));
                self.command_logger.log(&format!("Received: '{}'", response.trim()), Info);
                if !response.contains("+CMGS") {
                    self.logger.log("No '+CMGS' received after sending SMS.", Error);
                    return Err(io::Error::new(io::ErrorKind::Other, "no '+CMGS' received"));
                }
                try!(self.serial.read_line(&mut response));

                let start = time::precise_time_s();
                while time::precise_time_s() - start < 10f64 {
                    if try!(self.serial.read_line(&mut response)) != 0 {
                        break;
                    }
                }
                self.command_logger.log(&format!("Received: '{}'", response.trim()), Info);

                if response.trim() != "OK" {
                    self.logger.log("No 'OK' received after sending SMS.", Error);
                    return Err(io::Error::new(io::ErrorKind::Other, "no 'OK' received"));
                }
            } else {
                thread::sleep(Duration::from_secs(5));
            }

            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "GSM is off"))
        }
    }

    pub fn get_battery_status(&mut self) -> Result<(f64, f64), io::Error> {
        self.logger.log("Checking Battery status…", Info);
        if self.is_on() {
            let gsm_response = try!(self.send_command_read("AT+CBC"));
            let adc_response = try!(self.send_command_read("AT+CADC?"));

            if gsm_response.len() < 2 || adc_response.len() < 2 {
                self.logger.log("No response received when reading battery values.", Error);
                return Err(io::Error::new(io::ErrorKind::Other, "no response received"));
            }

            let mut response_found = false;
            let mut gsm_response_str = String::new();
            for line in gsm_response {
                if line.contains("+CBC:") {
                    response_found = true;
                    gsm_response_str = line;
                }
            }

            if response_found {
                let mut response_found = false;
                let mut adc_response_str = String::new();
                for line in adc_response {
                    if line.contains("+CADC:") {
                        response_found = true;
                        adc_response_str = line;
                    }
                }

                if response_found {
                    let gsm_response: Vec<&str> = gsm_response_str.split(",").collect();
                    let adc_response: Vec<&str> = adc_response_str.split(",").collect();

                    let gsm_voltage = match gsm_response[2].parse::<f64>() {
                        Ok(v) => v,
                        Err(e) => {
                            self.logger
                                .log(&format!("Invalid ADC battery check response: {:?}", e),
                                     Error);
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                      format!("invalid response received: {:?}",
                                                              e)));
                        }
                    };
                    let adc_voltage = match adc_response[1].parse::<f64>() {
                        Ok(v) => v,
                        Err(e) => {
                            self.logger
                                .log(&format!("Invalid ADC battery check response: {:?}", e),
                                     Error);
                            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                      format!("invalid response received: {:?}",
                                                              e)));
                        }
                    };

                    Ok(((gsm_voltage / 1000.0 - GSM_MIN_BAT) / (GSM_MAX_BAT - GSM_MIN_BAT),
                        (adc_voltage / 1000.0 - MAIN_MIN_BAT) / (MAIN_MAX_BAT - MAIN_MIN_BAT)))
                } else {
                    self.logger.log("Invalid ADC battery check response.", Error);
                    return Err(io::Error::new(io::ErrorKind::Other, "invalid response received"));
                }
            } else {
                self.logger.log("Invalid GSM battery check response.", Error);
                return Err(io::Error::new(io::ErrorKind::Other, "invalid response received"));
            }
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "GSM is off"))
        }
    }

    pub fn get_location(&self) -> Result<(f64, f64), io::Error> {
        // TODO
        Ok((0f64, 0f64))
    }

    fn send_command_read(&mut self, command: &str) -> Result<Vec<String>, io::Error> {
        try!(self.serial.get_mut().flush());
        try!(writeln!(self.serial.get_mut(), "{}", command));
        self.command_logger.log(&format!("Sent: '{}'", command), Info);

        let mut lines = Vec::new();
        let mut buf = String::new();
        while try!(self.serial.read_line(&mut buf)) != 0 {
            if buf.ends_with("\n") {
                buf.pop();
                if buf.ends_with("\r") {
                    buf.pop();
                }
            }
            self.command_logger.log(&format!("Received: '{}'", buf), Info);
            lines.push(buf.clone());
        }

        Ok(lines)
    }
}
