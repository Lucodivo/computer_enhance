// Casey Muratori's Computer Enhance random number generator ported to rust

fn rotate_left(v: u64, n: u64) -> u64 {
    (v << n) | (v >> (64 - n))
}

pub struct RandomSeries{
    a: u64,
    b: u64,
    c: u64, 
    d: u64
}

pub fn random_u64(random_series: &mut RandomSeries) -> u64 {
    let mut a = random_series.a;
    let mut b = random_series.b;
    let mut c = random_series.c;
    let mut d = random_series.d;

    let e = a.wrapping_sub(rotate_left(b, 27));

    a = b ^ rotate_left(c, 17);
    b = c.wrapping_add(d);
    c = d.wrapping_add(e);
    d = e.wrapping_add(a);

    random_series.a = a;
    random_series.b = b;
    random_series.c = c;
    random_series.d = d;

    d
}

pub fn seed(value: u64) -> RandomSeries {
    let mut random_series = RandomSeries{
        a: 0xf1ea5eedu64, 
        b: value, 
        c: value, 
        d: value};

    for _ in 0..20 {
        random_u64(&mut random_series);
    }

    random_series
}

pub fn random_in_range(random_series: &mut RandomSeries, min: f64, max: f64) -> f64 {
    let t = random_u64(random_series) as f64 / std::u64::MAX as f64;
    ((1.0 - t) * min) + (t * max)
}