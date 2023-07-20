use std::fs;
use std::env;
use json_parser::JsonValue;
use haversine_gen::read_haversine_binary_file;

mod utils;
use utils::{ StopWatch, printable_large_num };

struct PointPair{ x0: f64, y0: f64, x1: f64, y1: f64 }

fn main() {
    let mut stopwatch = StopWatch::now();

    let args: Vec<String> = env::args().collect();

    // TODO: Add print and performance options
    let usage = "\nUsage: \thaversine_gen [haversine_input.json]\n\
                          \thaversine_gen [haversine_input.json] [answers.f64]\n";

    assert!(args.len() >= 2 && args.len() <= 3, "{}", usage);
    
    let input_filename = args[1].parse::<String>().unwrap();
    let setup_clocks = stopwatch.lap_clocks();

    let json_bytes = fs::read(input_filename).expect("Failed to read json input file.");
    let read_file_clocks = stopwatch.lap_clocks();

    let json_root = json_parser::parse_json_bytes(&json_bytes).expect("Failed to parse json input file.");
    let parse_json_clocks = stopwatch.lap_clocks();

    let point_pairs = pairs_from_root_json(&json_root);
    let pull_point_pairs_clocks = stopwatch.lap_clocks();
    
    let haversine_vals = point_pairs.iter().map(|pp| haversine::haversine(pp.x0, pp.y0, pp.x1, pp.y1, None)).collect::<Vec<f64>>();
    let haversine_clocks = stopwatch.lap_clocks();

    let pair_count_f64 = point_pairs.len() as f64;
    let haversine_mean = haversine_vals.iter().fold(0.0_f64, |acc, &x| acc + (x / pair_count_f64));
    let haversine_mean_clocks = stopwatch.lap_clocks();

    let final_clocks = stopwatch.total_clocks();

    println!("Input file size:\t{} bytes", printable_large_num(json_bytes.len() as u64));
    println!("Pair count:\t\t{}", printable_large_num(point_pairs.len() as u64));
    println!("Haversine mean:\t\t{}\n", haversine_mean);
    println!("Total time:\t\t{:2}ms (CPU freq {}Hz)", stopwatch.total_secs() * 1_000_f64, printable_large_num(stopwatch.cpu_freq));
    println!("Total clocks:\t\t{}", printable_large_num(final_clocks));
    println!("Startup:\t\t{}\t\t({:.2}%)", printable_large_num(setup_clocks), setup_clocks as f64 / final_clocks as f64 * 100.0);
    println!("Read:\t\t\t{}\t({:.2}%)", printable_large_num(read_file_clocks), read_file_clocks as f64 / final_clocks as f64 * 100.0);
    println!("Generic JSON Parse:\t{}\t({:.2}%)", printable_large_num(parse_json_clocks), parse_json_clocks as f64 / final_clocks as f64 * 100.0);
    println!("Generic JSON to Pairs:\t{}\t({:.2}%)", printable_large_num(pull_point_pairs_clocks), pull_point_pairs_clocks as f64 / final_clocks as f64 * 100.0);
    println!("JSON to Pairs (TOTAL):\t{}\t({:.2}%)", printable_large_num(pull_point_pairs_clocks + parse_json_clocks), (pull_point_pairs_clocks + parse_json_clocks) as f64 / final_clocks as f64 * 100.0);
    println!("Haversine:\t\t{}\t({:.2}%)", printable_large_num(haversine_clocks), haversine_clocks as f64 / final_clocks as f64 * 100.0);
    println!("Mean:\t\t\t{}\t({:.2}%)", printable_large_num(haversine_mean_clocks), haversine_mean_clocks as f64 / final_clocks as f64 * 100.0);

    if args.len() == 3 {
        let answers_filename = args[2].parse::<String>().unwrap();
        match read_haversine_binary_file(&answers_filename) {
            Ok((answer_haversine_vals, answer_haversine_mean)) => {
                assert!(haversine_vals.len() == answer_haversine_vals.len(), 
                    "ERROR: Error data json had {} point pairs but answers file only had {} values.", 
                    haversine_vals.len(), answer_haversine_vals.len());
                if haversine_mean == answer_haversine_mean { 
                    println!("Calculated haversine mean matches the answers file!");
                } else { eprintln!("ERROR: Calculated haversine mean does *NOT* match the answers file."); }
            },
            Err(e) => {
                panic!("Failed to read haversine answers file: {}", e);
            }
        }
    }
}

fn pairs_from_root_json(json_root: &JsonValue) -> Vec<PointPair> {
    let mut point_pairs = Vec::<PointPair>::new();

    if let JsonValue::Object{ key_val_pairs: kvp } = json_root {
        if let (_, JsonValue::Array{ elements: json_point_pairs }) = kvp.iter().find(|&kv| kv.0 == b"pairs").expect("Couldn't find pairs in json.") {
            for json_point_pair in json_point_pairs {
                if let JsonValue::Object{ key_val_pairs: kvp } = json_point_pair {
                    let get_float = |key: &[u8]| -> f64 {
                        if let JsonValue::Number(n) = kvp.iter().find(|&kv| kv.0 == key).expect("Point in json is missing values.").1 {
                            return n;
                        } else { panic!("Point in json has non-number values.") }
                    };
                    point_pairs.push(PointPair{ 
                        x0: get_float(b"x0"),
                        y0: get_float(b"y0"),
                        x1: get_float(b"x1"),
                        y1: get_float(b"y1")
                    });
                }
            }
        }
    }

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