use std::fs::File;
use std::io::prelude::*;

pub fn read_haversine_binary_file(filename: &str) -> std::io::Result<(Vec<f64>, f64)> {
    let mut reader = File::open(filename)?;
    let file_size = reader.metadata()?.len();
    assert!(file_size > 0, "ERROR: Answers file is empty.");
    assert_eq!(file_size % 8, 0, "ERROR: Answers file size is not a multiple of 8 bytes (size of f64).");
    let answers_count = (file_size / 8) - 1; // last element is the mean

    let mut haversine_vals: Vec<f64> = Vec::<f64>::with_capacity(answers_count as usize);
    let mut buffer: [u8; 8] = [0; 8];
    for _ in 0..answers_count {
        reader.read(&mut buffer)?;
        let haversine_val = f64::from_le_bytes(buffer);
        haversine_vals.push(haversine_val);
    }

    reader.read(&mut buffer)?;
    let haversine_mean = f64::from_le_bytes(buffer);

    Ok((haversine_vals, haversine_mean))
}