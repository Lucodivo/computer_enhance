#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use std::fs;
use std::env;
use json_parser::{Json, JsonValue};
use haversine_gen::parse_haversine_binary_file;

use profiler::*;

struct PointPair{ x0: f64, y0: f64, x1: f64, y1: f64 }

fn main() {
    profiler_setup();

    let args: Vec<String> = env::args().collect();

    // TODO: Add print and performance options
    let usage = "\nUsage: \thaversine_gen [haversine_input.json]\n\
                          \thaversine_gen [haversine_input.json] [answers.f64]\n";

    let input_filename = args[1].parse::<String>().unwrap();
    assert!(args.len() >= 2 && args.len() <= 3, "{}", usage);
    
    time_assignments!(
    let json_bytes = fs::read(input_filename).expect("Failed to read json input file.");
    let json = json_parser::parse_json_bytes(&json_bytes).expect("Failed to parse json input file.");
    let point_pairs = pairs_from_root_json(&json);
    let haversine_vals = point_pairs.iter().map(|pp| haversine::haversine(pp.x0, pp.y0, pp.x1, pp.y1, None)).collect::<Vec<f64>>();
    let pair_count_f64 = point_pairs.len() as f64;
    let haversine_mean = haversine_vals.iter().fold(0.0_f64, |acc, &x| acc + (x / pair_count_f64));
    );

    if args.len() == 3 {
        time_block!("Check answers file");
        let answers_filename = args[2].parse::<String>().unwrap();
        match parse_haversine_binary_file(&answers_filename) {
            Ok((answer_haversine_vals, answer_haversine_mean)) => {
                assert!(haversine_vals.len() == answer_haversine_vals.len(), 
                    "ERROR: Error data json had {} point pairs but answers file only had {} values.", 
                    haversine_vals.len(), answer_haversine_vals.len());
                if haversine_mean != answer_haversine_mean { eprintln!("ERROR: Calculated haversine mean does *NOT* match the answers file."); }
            },
            Err(e) => {
                panic!("Failed to read haversine answers file: {}", e);
            }
        }
    }

    profiler_teardown();
}

fn pairs_from_root_json(json: &Json) -> Vec<PointPair> {
    time_function!();
    
    let mut point_pairs = Vec::<PointPair>::new();
        
    let mut root_object_context = json.get_root_context();
    if let Some((b"pairs", pairs_array_value)) = json.get_next_element(&mut root_object_context) {
        let mut pairs_array_context = json.open_collection(pairs_array_value);
        while let Some((_, pair_object_value)) = json.get_next_element(&mut pairs_array_context) {
            let mut pair_object_context = json.open_collection(pair_object_value);
            if let (Some((b"x0", JsonValue::Number(x0))),
                    Some((b"y0", JsonValue::Number(y0))),
                    Some((b"x1", JsonValue::Number(x1))),
                    Some((b"y1", JsonValue::Number(y1)))) = 
                    (json.get_next_element(&mut pair_object_context),
                    json.get_next_element(&mut pair_object_context),
                    json.get_next_element(&mut pair_object_context),
                    json.get_next_element(&mut pair_object_context)) {
                point_pairs.push(PointPair{ x0, y0, x1, y1 });
            } else {
                panic!("Expected four specific number member values from pair objects.");
            }
        }
    } else { panic!("Expected object to have pairs array."); }

    return point_pairs;
}

#[allow(dead_code)]
fn print_debug(point_pairs: &Vec<PointPair>, haversine_vals: &Vec<f64>, haversine_mean: f64) {
    
    println!("Point pairs:");
    for (i,pp) in point_pairs.iter().enumerate() {
        println!("{} - (x0: {}, y0: {}), (x1: {}, y1: {})", i+1, pp.x0, pp.y0, pp.x1, pp.y1);
    }
    println!("Haversine values:");
    for (i,hv) in haversine_vals.iter().enumerate() {
        println!("{} - {}", i+1, hv);
    }
    println!("Haversine mean: {}", haversine_mean);
}