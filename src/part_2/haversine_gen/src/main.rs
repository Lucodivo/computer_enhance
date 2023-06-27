use std::env;
use std::fs::File;
use std::io::prelude::*;

use rand_casey::{seed, random_in_range, RandomSeries};
use haversine::haversine;

struct PointPair{x0: f64, y0: f64, x1: f64, y1: f64}

fn main() {
    let args: Vec<String> = env::args().collect();

    let usage = "Usage: haversine_gen [uniform/cluster] [random seed] [num coords to gen]";

    assert_eq!(args.len(), 4, "{}", usage);
    
    //let dir = &args[0];
    let gen_type = args[1].parse::<String>().unwrap().to_lowercase();
    let seed = args[2].parse::<u64>().unwrap();
    let num_pairs = args[3].parse::<u64>().unwrap();

    let point_pairs;
    if gen_type == "uniform"  {
        point_pairs = gen_coords_uniform(seed, num_pairs);
    } else if gen_type == "cluster" { // cluster
        point_pairs = gen_coords_cluster(seed, num_pairs);
    } else {
        panic!("{}\ngeneration type value was {}", usage, gen_type);
    }

    write_point_pairs_json(&point_pairs).expect("Failed to write data to json file.");

    let haversine_mean = calc_haversine_mean(&point_pairs);
    
    println!("\
        Method: {}\n\
        Random Seed: {}\n\
        Pair Count: {}\n\
        Haversine Mean: {}", 
        gen_type, seed, num_pairs, haversine_mean);
    
}

fn calc_haversine_mean(point_pairs: &Vec<PointPair>) -> f64 {
    let mut haversine_mean: f64 = 0.0;
    let count = point_pairs.len() as f64;
    for point_pair in point_pairs {
        haversine_mean += haversine(point_pair.x0, point_pair.y0, point_pair.x1, point_pair.y1, None) / count;
    }
    haversine_mean
}

fn write_point_pairs_json(point_pairs: &Vec<PointPair>) -> std::io::Result<()> {

    let mut f = File::create(format!("data_{}_flex.json", point_pairs.len()))?;
    
    // header
    write!(f, "{{\"pairs\":[")?;
    
    if !point_pairs.is_empty() {

        // Handle first element separately as to not have a trailing comma (json doesn't accept that)
        let first_pair = &point_pairs[0];
        write!(f, "\n\t{{\"x0\":{},\"y0\":{},\"x1\":{},\"y1\":{}}}", first_pair.x0, first_pair.y0, first_pair.x1, first_pair.y1)?;
        
        for point_pair in &point_pairs[1..] {
            write!(f, ",\n\t{{\"x0\":{},\"y0\":{},\"x1\":{},\"y1\":{}}}", point_pair.x0, point_pair.y0, point_pair.x1, point_pair.y1)?;
        }
    }

    // footer
    writeln!(f, "\n]}}")?;

    return Result::Ok(());
}

fn rand_point_pair_in_range(random_series: &mut RandomSeries, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PointPair {
    PointPair{
        x0: random_in_range(random_series, min_x, max_x),
        y0: random_in_range(random_series, min_y, max_y),
        x1: random_in_range(random_series, min_x, max_x),
        y1: random_in_range(random_series, min_y, max_y)
    }
}

fn gen_coords_uniform(random_seed: u64, num_pairs: u64) -> Vec<PointPair> {
    
    let mut point_pairs = Vec::<PointPair>::with_capacity(num_pairs as usize);
    let mut random_series = seed(random_seed);

    for _ in 0..num_pairs {
        point_pairs.push(
            rand_point_pair_in_range(&mut random_series, -180.0, -90.0, 180.0, 90.0)
        );
    }

    return point_pairs;
}

fn gen_coords_cluster(random_seed: u64, num_pairs: u64) -> Vec<PointPair> {

    let mut point_pairs = Vec::<PointPair>::with_capacity(num_pairs as usize);
    let mut random_series = seed(random_seed);

    fn push_cluster_coords(point_pairs: &mut Vec<PointPair>, random_series: &mut RandomSeries, count: u64) {
        let rand_point_pair = rand_point_pair_in_range(random_series, -180.0, -90.0, 180.0, 90.0);

        let min_x = rand_point_pair.x0.min(rand_point_pair.x1);
        let max_x = rand_point_pair.x0.max(rand_point_pair.x1);
        let min_y = rand_point_pair.y0.min(rand_point_pair.y1);
        let max_y = rand_point_pair.y0.max(rand_point_pair.y1);
        
        for _ in 0..count {
            point_pairs.push(
                rand_point_pair_in_range(random_series, min_x, min_y, max_x, max_y)
            );
        }
    }

    let num_pair_partitions = 5;
    let size_of_clusters = num_pairs / num_pair_partitions;
    let size_of_last_cluster = num_pairs % num_pair_partitions;

    for _ in 0..num_pair_partitions {
        push_cluster_coords(&mut point_pairs, &mut random_series, size_of_clusters);
    }
    push_cluster_coords(&mut point_pairs, &mut random_series, size_of_last_cluster);

    return point_pairs;
}