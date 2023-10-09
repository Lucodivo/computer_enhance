#[allow(unused_imports)]
use profiler::*;

fn square(x: f64) -> f64 { 
    x * x 
}

fn radians(degrees: f64) -> f64 { 
    degrees * (std::f64::consts::PI / 180.0) 
}

// EarthRadius is generally expected to be 6372.8
pub fn haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: Option<f64>) -> f64 {
    // time_function!();
    
    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let d_lat = radians(lat2 - lat1);
    let d_lon = radians(lon2 - lon1);
    let lat1 = radians(lat1);
    let lat2 = radians(lat2);

    let a = square((d_lat / 2.0).sin()) + lat1.cos() * lat2.cos() * square((d_lon / 2.0).sin());
    let c = 2.0 * (a.sqrt()).asin();

    earth_radius.unwrap_or(6372.8) * c
}