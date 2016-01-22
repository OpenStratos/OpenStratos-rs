use std::fs::File;
use std::path::PathBuf;
use std::io::{Write, Error};
use std::ffi::OsStr;

use time;
use log;

pub struct Logger {
    file: File,
    prefix: &'static str,
}

impl Logger {
    pub fn new<P: AsRef<OsStr> + ?Sized>(path: &P,
                                         filename: &str,
                                         prefix: &'static str)
                                         -> Result<Logger, Error> {
        let mut pathbuf = PathBuf::from(path);
        pathbuf.set_file_name(format!("{}.{}",
                                      filename,
                                      time::now_utc()
                                          .strftime("%F.%H-%M-%S")
                                          .unwrap()));
        pathbuf.set_extension(".log");
        let path = pathbuf.as_path();

        Ok(Logger {
            file: try!(File::create(path)),
            prefix: prefix,
        })
    }

    pub fn log(&mut self, message: &str, level: log::LogLevel) {
        let log_message = format!("[{}][{}] - {} - {}",
                                  self.prefix,
                                  level,
                                  time::now_utc()
                                      .strftime("%D %T.%f")
                                      .unwrap(),
                                  message);
        if let Err(e) = self.file.write_all(&log_message.into_bytes()) {
            error!("Error when writing a log message: {}", e)
        }
    }
}
