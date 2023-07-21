use std::fs;
use profiler::*;

const ELEMENT_SIZE: usize = std::mem::size_of::<f64>();

pub fn read_haversine_binary_file(filename: &str) -> std::io::Result<(Vec<f64>, f64)> {
    time_function!();
    let file_bytes = fs::read(filename)?;
    let file_size = file_bytes.len();
    assert!(file_size > 0, "ERROR: Answers file is empty.");
    assert_eq!(file_size % ELEMENT_SIZE, 0, "ERROR: Answers file size is not a multiple of 8 bytes (size of f64).");
    Ok((parse_haversine_answers(&file_bytes), parse_haversine_mean(&file_bytes)))
}

fn parse_haversine_answers(file_bytes: &[u8]) -> Vec<f64> {
    time_function!();
    let haversine_vals_end = file_bytes.len() - ELEMENT_SIZE; // last element is the mean
    let mut haversine_vals: Vec<f64> = Vec::<f64>::with_capacity(haversine_vals_end / ELEMENT_SIZE);
    for i in (0..haversine_vals_end).step_by(ELEMENT_SIZE) {
        let haversine_bytes: [u8; 8] = file_bytes[i..i + ELEMENT_SIZE].try_into().expect("Failed to convert [u8] to [u8; 8]");
        let haversine_val = f64::from_le_bytes(haversine_bytes);
        haversine_vals.push(haversine_val);
    }
    haversine_vals
}

fn parse_haversine_mean(file_bytes: &[u8]) -> f64 {
    time_function!();
    let mean_bytes: [u8; ELEMENT_SIZE] = file_bytes[(file_bytes.len() - ELEMENT_SIZE)..file_bytes.len()].try_into().expect("Failed to convert [u8] to [u8; 8]");
    f64::from_le_bytes(mean_bytes)
}