#![allow(dead_code)]
use clocks::{ read_cpu_timer, measure_cpu_freq, clocks_to_secs };

pub struct StopWatch { creation_stamp: u64, lap_stamp: u64, pub cpu_freq: u64 }
impl StopWatch {
    pub fn now() -> StopWatch { // TODO: Let caller specify measurement time?
        let cpu_freq = measure_cpu_freq(100);
        let now = read_cpu_timer();
        StopWatch{ creation_stamp: now,
                    lap_stamp: now,
                    cpu_freq: cpu_freq } 
    }
    pub fn lap_clocks(&mut self) -> u64 {
        let now = read_cpu_timer();
        let lap = now - self.lap_stamp;
        self.lap_stamp = now;
        return lap;
    }
    pub fn total_clocks(&self) -> u64 { read_cpu_timer() - self.creation_stamp }
    pub fn total_secs(&self) -> f64 { clocks_to_secs(self.total_clocks(), self.cpu_freq) }
}

pub fn printable_freq(freq: u64) -> String {
    match freq {
        0..=999 => format!("{} Hz", freq),
        1_000..=999_999 => format!("{:.3} KHz", (freq as f64) / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.2} MHz", (freq as f64) / 1_000_000.0),
        1_000_000_000..=999_999_999_999 => format!("{:.1} GHz", (freq as f64) / 1_000_000_000.0),
        _ => format!("{:.1} THz", (freq as f64) / 1_000_000_000_000.0)
    }
}

pub fn printable_large_num(number: u64) -> String {
    number.to_string()
        .as_bytes()
        .rchunks(3).rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}