use std::fs;
use std::env;
use json_parser::JsonValue;
use haversine_gen::read_haversine_binary_file;

mod utils;
use utils::StopWatch;

struct PointPair{ x0: f64, y0: f64, x1: f64, y1: f64 }

fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: Add print and performance options
    let usage = "\nUsage: \thaversine_gen [haversine_input.json]\n\
                          \thaversine_gen [haversine_input.json] [answers.f64]\n";

    assert!(args.len() >= 2 && args.len() <= 3, "{}", usage);
    
    let input_filename = args[1].parse::<String>().unwrap();

    let mut stopwatch = StopWatch::now();
    let json_str = fs::read_to_string(input_filename).expect("Failed to read json input file.");
    println!("Reading json file took {} ms", stopwatch.lap_millis());

    let json_root = json_parser::parse_json_str(&json_str).expect("Failed to parse json input file.");
    println!("Parsing json file took {} ms", stopwatch.lap_millis());

    let point_pairs = pairs_from_root_json(&json_root);
    println!("Extracting point pairs from json took {} ms", stopwatch.lap_millis());
    
    let haversine_vals = point_pairs.iter().map(|pp| haversine::haversine(pp.x0, pp.y0, pp.x1, pp.y1, None)).collect::<Vec<f64>>();
    println!("Calculating haversine values took {} ms", stopwatch.lap_millis());

    let pair_count_f64 = point_pairs.len() as f64;
    let haversine_mean = haversine_vals.iter().fold(0.0_f64, |acc, &x| acc + (x / pair_count_f64));

    //print_debug(&point_pairs, &haversine_vals, haversine_mean);

    if args.len() == 3 {
        let answers_filename = args[2].parse::<String>().unwrap();
        match read_haversine_binary_file(&answers_filename) {
            Ok((answer_haversine_vals, answer_haversine_mean)) => {
                assert_eq!(haversine_vals.len(), answer_haversine_vals.len());
                assert!(haversine_vals.iter().eq(answer_haversine_vals.iter()));
                println!("Haversine values match the answers file!");
                assert_eq!(haversine_mean, answer_haversine_mean);
                println!("Haversine mean matches the answers file!");
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
        if let (_, JsonValue::Array{ elements: json_point_pairs }) = kvp.iter().find(|&kv| kv.0 == "pairs").expect("Couldn't find pairs in json.") {
            for json_point_pair in json_point_pairs {
                if let JsonValue::Object{ key_val_pairs: kvp } = json_point_pair {
                    let get_float = |key: &str| -> f64 {
                        if let JsonValue::Number(n) = kvp.iter().find(|&kv| kv.0 == key).expect("Point in json is missing values.").1 {
                            return n;
                        } else { panic!("Point in json has non-number values.") }
                    };
                    point_pairs.push(PointPair{ 
                        x0: get_float("x0"),
                        y0: get_float("y0"),
                        x1: get_float("x1"),
                        y1: get_float("y1")
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