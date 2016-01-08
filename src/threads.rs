use State;

use gsm::Gsm;

use std::thread;
use std::sync::Mutex;
use std::time::Duration;

use log;
use fern;
use time;

use log::LogLevel::Info;
use fern::IntoLog;

pub fn system(state: &Mutex<State>) {
    println!("Hello from system thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}

pub fn battery(state: &Mutex<State>, gsm: &Gsm) {
    let log_path = format!("data/logs/GSM/Battery.{}.log",
                           time::now_utc()
                               .strftime("%Y-%m-%d.%H-%M-%S")
                               .unwrap());
    let logger = fern::DispatchConfig {
                     format: Box::new(|msg: &str,
                                       level: &log::LogLevel,
                                       _location: &log::LogLocation| {
                         format!("[{}][{}] {}",
                                 time::now_utc().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                                 level,
                                 msg)
                     }),
                     output: vec![fern::OutputConfig::file(&log_path)],
                     level: log::LogLevelFilter::Info,
                 }
                 .into_fern_logger()
                 .unwrap(); // TODO better error handling (end thread with error log?)

    while {
        let state = state.lock().unwrap();
        *state != State::ShutDown
    } {
        // TODO better error handling
        if gsm.is_on().unwrap() {
            let (main_battery, gsm_battery) = gsm.get_battery_status().unwrap();
            // logger.log(format!("[MAIN] {}", main_battery), Info);
            // logger.log(format!("[GSM] {}", gsm_battery).as_ref(),
            //            &Info,
            //            _);
        } else {
            thread::sleep(Duration::from_secs(15 * 60));
            gsm.turn_on();

            let (main_battery, gsm_battery) = gsm.get_battery_status().unwrap();
            // logger.log(format!("[MAIN] {}", main_battery), Info);
            // logger.log(format!("[GSM] {}", gsm_battery), Info);

            gsm.turn_off();
        }

        thread::sleep(Duration::from_secs(3 * 30));
    }
}

pub fn pictures(state: &Mutex<State>) {
    println!("Hello from pictures thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}
