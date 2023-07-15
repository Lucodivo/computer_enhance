use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "Usage: clocks [measure time in ms]");
    
    // let dir = &args[0];
    let ms = args[1].parse::<u64>().unwrap();

    clocks::testing(ms);
}