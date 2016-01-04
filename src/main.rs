extern crate time;
#[macro_use]
extern crate log;
extern crate fern;

mod threads;

use std::result::Result;
use std::{fs, io};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

const STATE_FILE: &'static str = "data/last_state.txt";

#[derive(Debug, Clone, Copy)]
pub enum State {
    Initializing,
    AcquiringFix,
    FixAquired,
    WaitingLaunch,
    GoingUp,
    GoingDown,
    Landed,
    ShutDown,
    SafeMode,
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

fn main_logic() {
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

    if cfg!(feature = "debug") {
        println!("[OpenStratos] Logger started.");
    }
    // TODO log the version of OpenStratos
    info!("Logging started.");

    debug!("Starting system thread…");
    let system_state = shared_state.clone();
    let system_thread = thread::spawn(move || {
        threads::system(&system_state);
    });
    debug!("System thread started.");

    // TODO initialize(&logger, now);

    debug!("Starting battery thread…");
    let battery_state = shared_state.clone();
    let battery_thread = thread::spawn(move || {
        threads::battery(&battery_state);
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

    debug!("Joining threads…");
    match picture_thread.join() {
        Ok(_) => trace!("Picture thread joined."),
        Err(e) => {
            error!("Picture thread panicked! {:?}", e)
        }
    }
    match battery_thread.join() {
        Ok(_) => trace!("Battery thread joined."),
        Err(e) => {
            error!("Battery thread panicked! {:?}", e)
        }
    }
    match system_thread.join() {
        Ok(_) => trace!("System thread joined."),
        Err(e) => {
            error!("System thread panicked! {:?}", e)
        }
    }
    debug!("Threads joined.");

    // TODO shut_down(&logger);
}

fn safe_mode() {
    // TODO
}

fn set_state(state: State) -> Result<State, io::Error> {
    let mut f = try!(fs::File::create(STATE_FILE));
    try!(f.write_all(&format!("{:?}", state).into_bytes()[..]));

    Ok(state)
}

fn modify_shared_state(st: State, shared: &Mutex<State>) -> Result<State, io::Error> {
    match set_state(st) {
        Err(e) => Err(e),
        Ok(st) => {
            let mut state = match shared.lock() {
                Ok(st) => st,
                Err(poisoned) => {
                    error!("The state mutex has been poisoned!");
                    poisoned.into_inner()
                },
            };
            *state = st;
            info!("State changed to {:?}.", st);

            Ok(st)
        }
    }
}

#[inline]
fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

#[inline]
fn check_or_create(path: &str) {
    if !path_exists(path) {
        fs::create_dir(path).unwrap()
    }
}
