use std::io;

pub struct Gsm;

impl Gsm {
    pub fn is_on(&self) -> Result<bool, io::Error> {
        // TODO
        Ok(true)
    }

    pub fn turn_on(&mut self) -> Result<(), io::Error> {
        // TODO
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), io::Error> {
        // TODO
        Ok(())
    }

    pub fn get_battery_status(&self) -> Result<(f64, f64), io::Error> {
        // TODO
        Ok((0f64, 0f64))
    }
}
