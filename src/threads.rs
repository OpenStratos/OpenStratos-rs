use State;

use gsm::Gsm;
use logger::Logger;

use std::{fs, thread};
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

use time;
use log::LogLevel;

pub fn system(state: &Mutex<State>) {
    println!("Hello from system thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}

pub fn battery(state: &Mutex<State>, gsm: &Mutex<Gsm>) {
    let mut logger = Logger::new("data/logs/GSM", "Battery", "Battery").unwrap();

    while {
        let state = state.lock().unwrap();
        *state != State::ShutDown
    } {
        let mut gsm = gsm.lock().unwrap();
        let (main_battery, gsm_battery) = if gsm.is_on() {
            match gsm.get_battery_status() {
                Err(e) => {
                    error!("Error reading battery status! {:?}", e);
                    thread::sleep(Duration::from_secs(3 * 30));
                    continue;
                }
                Ok((main, gsm)) => (main, gsm),
            }
        } else {
            thread::sleep(Duration::from_secs(15 * 60));

            gsm.turn_on();
            let (main_battery, gsm_battery) = match gsm.get_battery_status() {
                Err(e) => {
                    error!("Error reading battery status! {:?}", e);
                    thread::sleep(Duration::from_secs(3 * 30));
                    continue;
                }
                Ok((main, gsm)) => (main, gsm),
            };
            gsm.turn_off();

            (main_battery, gsm_battery)
        };

        logger.log(format!("[MAIN] {}", main_battery).as_ref(), LogLevel::Info);
        logger.log(format!("[GSM] {}", gsm_battery).as_ref(), LogLevel::Info);

        thread::sleep(Duration::from_secs(3 * 30));
    }
}

pub fn pictures(state: &Mutex<State>) {
    println!("Hello from pictures thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}
