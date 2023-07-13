use std::fs;
use std::env;
use json_parser::JsonValue;
use haversine_gen::read_haversine_binary_file;

struct PointPair{ x0: f64, y0: f64, x1: f64, y1: f64 }

fn main() {
    let args: Vec<String> = env::args().collect();

    let usage = "\nUsage: \thaversine_gen [haversine_input.json]\n\thaversine_gen [haversine_input.json] [answers.f64]\n";

    assert!(args.len() >= 2 && args.len() <= 3, "{}", usage);
    
    let input_filename = args[1].parse::<String>().unwrap();

    let json_str = fs::read_to_string(input_filename).expect("Failed to read json input file.");

    let json_root = json_parser::parse_json_str(&json_str).expect("Failed to parse json input file.");

    let point_pairs = pairs_from_root_json(&json_root);
    let haversine_vals = point_pairs.iter().map(|pp| haversine::haversine(pp.x0, pp.y0, pp.x1, pp.y1, None)).collect::<Vec<f64>>();
    let pair_count_f64 = point_pairs.len() as f64;
    let haversine_mean = haversine_vals.iter().fold(0.0_f64, |acc, &x| acc + (x / pair_count_f64));
    
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