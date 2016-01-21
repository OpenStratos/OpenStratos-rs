use std::{io, thread};
use std::io::Write;
use std::fs::File;
use std::time::Duration;

use wiringpi;
use serial;
use time;
use log::LogLevel::*;

use logger::Logger;

use serial::posix::TTYPort;
use wiringpi::pin::{InputPin, OutputPin, Value};

const GSM_SERIAL: &'static str = "/dev/ttyUSB0";

pub struct Gsm {
    serial: TTYPort,
    logger: Logger,
    #[cfg(feature = "debug")]
    command_logger: Logger,
    power_pin: OutputPin<wiringpi::pin::WiringPi>,
    status_pin: InputPin<wiringpi::pin::WiringPi>,
}

impl Gsm {
    #[cfg(not(feature = "debug"))]
    pub fn initialize(wiring_pi: &wiringpi::WiringPi<wiringpi::pin::WiringPi>)
                      -> Result<Gsm, io::Error> {
        Ok(Gsm {
            serial: try!(serial::open(GSM_SERIAL)),
            logger: try!(Logger::new("data/logs/GSM", "GSM", "GSM")),
            power_pin: wiring_pi.output_pin(7),
            status_pin: wiring_pi.input_pin(21),
        })
    }

    #[cfg(feature = "debug")]
    pub fn initialize(wiring_pi: &wiringpi::WiringPi<wiringpi::pin::WiringPi>)
                      -> Result<Gsm, io::Error> {
        let log_path = format!("data/logs/GSM/GSM.{}.log",
                               time::now_utc()
                                   .strftime("%Y-%m-%d.%H-%M-%S")
                                   .unwrap());
        let logger = try!(File::create(log_path));

        let log_path = format!("data/logs/GSM/GSMCommands.{}.log",
                               time::now_utc()
                                   .strftime("%Y-%m-%d.%H-%M-%S")
                                   .unwrap());
        let command_logger = try!(File::create(log_path));

        Ok(Gsm {
            serial: try!(serial::open(GSM_SERIAL)),
            logger: logger,
            command_logger: command_logger,
            power_pin: wiring_pi.output_pin(7),
            status_pin: wiring_pi.input_pin(21),
        })
    }

    pub fn is_on(&self) -> bool {
        self.status_pin.digital_read() == Value::High
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

    pub fn get_battery_status(&self) -> Result<(f64, f64), io::Error> {
        // TODO
        Ok((0f64, 0f64))
    }
}
