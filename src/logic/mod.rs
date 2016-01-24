use std::{thread, fs};
use std::sync::{Arc, Mutex};

use {State, threads, wiringpi};
use utils::*;
use gsm::Gsm;

/// Main logic of OpenStratos
pub fn main_logic() {
    check_or_create("data");
    let shared_state = Arc::new(Mutex::new(State::set(State::Initializing).unwrap()));

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
    let wiring_pi = wiringpi::setup();
    let shared_gsm = Arc::new(Mutex::new(Gsm::initialize(&wiring_pi).unwrap()));

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

    State::modify_shared(State::AcquiringFix, &shared_state).unwrap();

    // TODO main_while(&logger, &state);

    State::modify_shared(State::ShutDown, &shared_state).unwrap();

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
