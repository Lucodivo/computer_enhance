#![allow(dead_code)]

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