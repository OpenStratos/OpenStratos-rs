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
use std::{fs, io};
use std::io::{Read, Write};
use std::sync::Mutex;

use logic::*;

const STATE_FILE: &'static str = "data/last_state.txt";

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
    fn from_str(text: &str) -> State {
        match text {
            "Initializing" => State::Initializing,
            "AcquiringFix" => State::AcquiringFix,
            "FixAcquired" => State::FixAcquired,
            "WaitingLaunch" => State::WaitingLaunch,
            "GoingUp" => State::GoingUp,
            "GoingDown" => State::GoingDown,
            "Landed" => State::Landed,
            "ShutDown" => State::ShutDown,
            "SafeMode" => State::SafeMode,
            _ => unreachable!(),
        }
    }

    pub fn get_last() -> Result<State, io::Error> {
        let mut f = try!(fs::File::open("foo.txt"));
        let mut buffer = String::new();
        try!(f.read_to_string(&mut buffer));

        Ok(State::from_str(buffer.trim()))
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Coordinates {
        Coordinates {latitude: latitude, longitude: longitude}
    }

    pub fn get_latitude(&self) -> f64 {self.latitude}
    pub fn get_longitude(&self) -> f64 {self.longitude}
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
