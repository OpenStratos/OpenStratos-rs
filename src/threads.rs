use State;

use std::sync::Mutex;

pub fn system(state: &Mutex<State>) {
    println!("Hello from system thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}

pub fn battery(state: &Mutex<State>) {
    println!("Hello from battery thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}

pub fn pictures(state: &Mutex<State>) {
    println!("Hello from pictures thread!");
    let state = state.lock().unwrap();
    println!("State: '{:?}'", *state);
}
