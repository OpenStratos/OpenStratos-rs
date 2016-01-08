extern crate time;
#[macro_use]
extern crate log;
extern crate fern;
extern crate serial;

mod threads;
mod gsm;

use std::result::Result;
use std::{fs, io};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use gsm::Gsm;

const STATE_FILE: &'static str = "data/last_state.txt";

#[derive(Debug, Clone, Copy, PartialEq)]
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

    if !path_exists(STATE_FILE) {
        if cfg!(feature = "debug") {
            println!("[OpenStratos] No state file. Starting main logic…");
        }
        main_logic();
    } else {
        if cfg!(feature = "debug") {
            println!("[OpenStratos] State file found. Starting safe mode…");
        }
        safe_mode();
    }

    if cfg!(feature = "power-off") {
        // TODO power off
    }
}

/// Main logic of OpenStratos
pub fn main_logic() {
    check_or_create("data");
    let shared_state = Arc::new(Mutex::new(set_state(State::Initializing).unwrap()));

    check_or_create("data/logs");
    check_or_create("data/logs/main");
    check_or_create("data/logs/system");
    check_or_create("data/logs/camera");
    check_or_create("data/logs/GPS");
    check_or_create("data/logs/GSM");

    if cfg!(feature = "debug") {
        println!("[OpenStratos] Starting logger…");
    }

    init_logger();

    if cfg!(feature = "debug") {
        println!("[OpenStratos] Logger started.");
    }
    info!("OpenStratos {}", env!("CARGO_PKG_VERSION"));
    info!("Logging started.");

    debug!("Starting system thread…");
    let system_state = shared_state.clone();
    let system_thread = thread::spawn(move || {
        threads::system(&system_state);
    });
    debug!("System thread started.");

    // TODO initialize(&logger, now);

    // TODO from initialize?
    // TODO better error handling
    let shared_gsm = Arc::new(Gsm::initialize().unwrap());

    debug!("Starting battery thread…");
    let battery_state = shared_state.clone();
    let gsm = shared_gsm.clone();
    let battery_thread = thread::spawn(move || {
        threads::battery(&battery_state, &gsm);
    });
    debug!("Battery thread started.");

    debug!("Starting pictures thread…");
    let picture_state = shared_state.clone();
    let picture_thread = thread::spawn(move || {
        threads::pictures(&picture_state);
    });
    debug!("Picture thread started.");

    modify_shared_state(State::AcquiringFix, &shared_state).unwrap();

    // TODO main_while(&logger, &state);

    modify_shared_state(State::ShutDown, &shared_state).unwrap();

    debug!("Joining threads…");
    if let Err(e) = picture_thread.join() {
        error!("Picture thread panicked! {:?}", e)
    }
    if let Err(e) = battery_thread.join() {
        error!("Battery thread panicked! {:?}", e)
    }
    if let Err(e) = system_thread.join() {
        error!("System thread panicked! {:?}", e)
    }
    debug!("Threads joined.");

    // TODO shut_down(&logger);
}

/// Safe mode of OpenStratos
pub fn safe_mode() {
    // TODO
    fs::remove_dir_all("data").unwrap()
}

/// Sets the current state of OpenStratos
pub fn set_state(state: State) -> Result<State, io::Error> {
    let mut f = try!(fs::File::create(STATE_FILE));
    try!(f.write_all(&format!("{:?}", state).into_bytes()[..]));

    Ok(state)
}

/// Modifies the shared state
pub fn modify_shared_state(st: State, shared: &Mutex<State>) -> Result<State, io::Error> {
    match set_state(st) {
        Err(e) => Err(e),
        Ok(st) => {
            let mut state = match shared.lock() {
                Ok(st) => st,
                Err(poisoned) => {
                    error!("The state mutex has been poisoned!");
                    // TODO panic if state is not high enough
                    poisoned.into_inner()
                },
            };
            *state = st;
            info!("State changed to {:?}.", st);

            Ok(st)
        }
    }
}

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn check_or_create(path: &str) {
    if !path_exists(path) {
        fs::create_dir(path).unwrap()
    }
}

fn init_logger() {
    let log_path = format!("data/logs/main/OpenStratos.{}.log",
                           time::now_utc()
                               .strftime("%Y-%m-%d.%H-%M-%S")
                               .unwrap());

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}",
                    time::now_utc().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                    level,
                    msg)
        }),
        output: if cfg!(feature = "debug") {
            vec![fern::OutputConfig::stdout(), fern::OutputConfig::file(&log_path)]
        } else {
            vec![fern::OutputConfig::file(&log_path)]
        },
        level: if cfg!(feature = "debug") {
            log::LogLevelFilter::Trace
        } else {
            log::LogLevelFilter::Info
        },
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
}
