#![allow(dead_code)]
use std::time::Instant;

pub struct StopWatch { start: Instant }
impl StopWatch {
    pub fn now() -> StopWatch { 
        StopWatch{ start: Instant::now() } 
    }
    pub fn lap_millis(&mut self) -> u128 {
        let elapsed_millis = self.start.elapsed().as_millis();
        self.start = Instant::now();
        return elapsed_millis;
    }
    pub fn lap_micros(&mut self) -> u128 {
        let elapsed_micros = self.start.elapsed().as_micros();
        self.start = Instant::now();
        return elapsed_micros;
    }
    pub fn lap_nanos(&mut self) -> u128 {
        let elapsed_nanos = self.start.elapsed().as_nanos();
        self.start = Instant::now();
        return elapsed_nanos;
    }
    pub fn reset(&mut self) { self.start = Instant::now(); }
}