extern crate time;
#[macro_use]
extern crate log;
extern crate fern;
extern crate serial;
extern crate wiringpi;

mod threads;
mod gsm;
mod logger;
mod utils;
mod logic;

use std::result::Result;
use std::str::FromStr;
use std::error::Error as StdError;
use std::{fs, io, fmt};
use std::io::{Read, Write};
use std::sync::Mutex;

use logic::*;

const STATE_FILE: &'static str = "data/last_state.txt";

#[derive(Debug)]
pub enum Error {
    ParseStateError(ParseStateError),
    IOError(io::Error),
}

impl From<ParseStateError> for Error {
    fn from(e: ParseStateError) -> Error {
        Error::ParseStateError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::ParseStateError(ref e) => e.description(),
            &Error::IOError(ref e) => e.description(),
        }
    }
}


#[derive(Debug)]
pub struct ParseStateError {
    description: String,
}

impl ParseStateError {
    fn new(s: &str) -> ParseStateError {
        ParseStateError { description: format!("Could not parse {} as a valid State", s) }
    }
}

impl fmt::Display for ParseStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl StdError for ParseStateError {
    fn description(&self) -> &str {
        self.description.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Initializing,
    AcquiringFix,
    FixAcquired,
    WaitingLaunch,
    GoingUp,
    GoingDown,
    Landed,
    ShutDown,
    SafeMode,
}

impl State {
    pub fn get_last() -> Result<State, Error> {
        let mut f = try!(fs::File::open("foo.txt"));
        let mut buffer = String::new();
        try!(f.read_to_string(&mut buffer));

        Ok(try!(State::from_str(buffer.trim())))
    }

    /// Sets the current state of OpenStratos
    pub fn set(state: State) -> Result<State, io::Error> {
        let mut f = try!(fs::File::create(STATE_FILE));
        try!(f.write_all(&format!("{:?}", state).into_bytes()[..]));

        Ok(state)
    }

    /// Modifies the shared state
    pub fn modify_shared(st: State, shared: &Mutex<State>) -> Result<State, io::Error> {
        match State::set(st) {
            Err(e) => Err(e),
            Ok(st) => {
                let mut state = match shared.lock() {
                    Ok(st) => st,
                    Err(poisoned) => {
                        error!("The state mutex has been poisoned!");
                        // TODO panic if state is not high enough
                        poisoned.into_inner()
                    }
                };
                *state = st;
                info!("State changed to {:?}.", st);

                Ok(st)
            }
        }
    }
}

impl FromStr for State {
    type Err = ParseStateError;
    fn from_str(s: &str) -> Result<State, ParseStateError> {
        match s {
            "Initializing" => Ok(State::Initializing),
            "AcquiringFix" => Ok(State::AcquiringFix),
            "FixAcquired" => Ok(State::FixAcquired),
            "WaitingLaunch" => Ok(State::WaitingLaunch),
            "GoingUp" => Ok(State::GoingUp),
            "GoingDown" => Ok(State::GoingDown),
            "Landed" => Ok(State::Landed),
            "ShutDown" => Ok(State::ShutDown),
            "SafeMode" => Ok(State::SafeMode),
            _ => Err(ParseStateError::new(s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Coordinates {
        Coordinates {
            latitude: latitude,
            longitude: longitude,
        }
    }

    pub fn get_latitude(&self) -> f64 {
        self.latitude
    }
    pub fn get_longitude(&self) -> f64 {
        self.longitude
    }
}

fn main() {
    if cfg!(feature = "debug") {
        if cfg!(feature = "sim") {
            println!("[OpenStratos] Simulation.");
        } else if cfg!(feature = "real-sim") {
            println!("[OpenStratos] Realistic simulation.");
        }

        println!("[OpenStratos] Starting…");
    }

    if !fs::metadata(STATE_FILE).is_ok() {
        if cfg!(feature = "debug") {
            println!("[OpenStratos] No state file. Starting main logic…");
        }
        logic::main_logic();
    } else {
        if cfg!(feature = "debug") {
            println!("[OpenStratos] State file found. Starting safe mode…");
        }
        logic::safe_mode();
    }

    if cfg!(feature = "power-off") {
        // TODO power off
    }
}
