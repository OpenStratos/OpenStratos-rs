use std::io;
use std::io::{Read, Write};
use serial;
use time;
use log;
use serial::posix::TTYPort;
use fern::{Logger, DispatchConfig, OutputConfig, IntoLog};

#[cfg(feature = "sms")]
const GSM_SERIAL: &'static str = "/dev/ttyUSB0";
#[cfg(not(feature = "sms"))]
const GSM_SERIAL: &'static str = "/dev/tty2"; // Example, to be able to run it

#[cfg(not(feature = "debug"))]
pub struct Gsm {
    serial: TTYPort,
    logger: Box<Logger>,
}

#[cfg(feature = "debug")]
pub struct Gsm {
    serial: TTYPort,
    logger: Box<Logger>,
    command_logger: Box<Logger>,
}

impl Gsm {
    #[cfg(not(feature = "debug"))]
    pub fn initialize() -> Result<Gsm, io::Error> {
        let log_path = format!("data/logs/GSM/GSM.{}.log",
                               time::now_utc()
                                   .strftime("%Y-%m-%d.%H-%M-%S")
                                   .unwrap());
        let logger = try!(DispatchConfig {
                              format: Box::new(|msg: &str,
                                                level: &log::LogLevel,
                                                _location: &log::LogLocation| {
                                  format!("[{}][{}] {}",
                                          time::now_utc().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                                          level,
                                          msg)
                              }),
                              output: vec![OutputConfig::file(&log_path)],
                              level: log::LogLevelFilter::Info,
                          }
                          .into_fern_logger());

        Ok(Gsm {
            serial: try!(serial::open(GSM_SERIAL)),
            logger: logger,
        })
    }

    #[cfg(feature = "debug")]
    pub fn initialize() -> Result<Gsm, io::Error> {
        let log_path = format!("data/logs/GSM/GSM.{}.log",
                               time::now_utc()
                                   .strftime("%Y-%m-%d.%H-%M-%S")
                                   .unwrap());

        let logger = try!(DispatchConfig {
                              format: Box::new(|msg: &str,
                                                level: &log::LogLevel,
                                                _location: &log::LogLocation| {
                                  format!("[{}][{}] {}",
                                          time::now_utc().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                                          level,
                                          msg)
                              }),
                              output: vec![OutputConfig::file(&log_path)],
                              level: log::LogLevelFilter::Info,
                          }
                          .into_fern_logger());

        let log_path = format!("data/logs/GSM/GSMCommands.{}.log",
                               time::now_utc()
                                   .strftime("%Y-%m-%d.%H-%M-%S")
                                   .unwrap());

        let command_logger = try!(DispatchConfig {
                                      format: Box::new(|msg: &str,
                                                        level: &log::LogLevel,
                                                        _location: &log::LogLocation| {
                                          format!("[{}][{}] {}",
                                                  time::now_utc()
                                                      .strftime("%Y-%m-%d][%H:%M:%S")
                                                      .unwrap(),
                                                  level,
                                                  msg)
                                      }),
                                      output: vec![OutputConfig::file(&log_path)],
                                      level: log::LogLevelFilter::Trace,
                                  }
                                  .into_fern_logger());

        Ok(Gsm {
            serial: try!(serial::open(GSM_SERIAL)),
            logger: logger,
            command_logger: command_logger,
        })
    }

    pub fn is_on(&self) -> Result<bool, io::Error> {
        // TODO
        Ok(true)
    }

    pub fn turn_on(&self) -> Result<(), io::Error> {
        // TODO
        Ok(())
    }

    pub fn turn_off(&self) -> Result<(), io::Error> {
        // TODO
        Ok(())
    }

    pub fn get_battery_status(&self) -> Result<(f64, f64), io::Error> {
        // TODO
        Ok((0f64, 0f64))
    }
}
