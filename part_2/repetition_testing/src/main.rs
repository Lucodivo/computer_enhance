struct RepetitionTestData {
    iterations: u64,
    total_clocks: u64,
    min_clocks: u64, 
    max_clocks: u64,
}

struct MeasureableActionResults {
    byte_count: u64,
}

trait MeasureableAction {
    fn action(&mut self) -> MeasureableActionResults;
    fn reset(&mut self);
    fn tag(&self) -> &'static str;
}

struct FSReadRepetition {
    file_bytes: Vec<u8>,
}

impl MeasureableAction for FSReadRepetition {
    fn action(&mut self) -> MeasureableActionResults {
        self.file_bytes = std::fs::read("../data/seed_123123/data_1000000_flex.json").expect("Couldn't fs::read file");
        MeasureableActionResults {
            byte_count: self.file_bytes.len() as u64,
        }
    }
    fn reset(&mut self) {
        // this deallocates anything previously held on by file_bytes
        // .clear() would not deallocate
        self.file_bytes = Vec::new();
    }
    fn tag(&self) -> &'static str{ "fs::read" }
}
fn main() {
    let cpu_freq = clocks::measure_cpu_freq(1000);

    let max_iterations: u64 = 10;
    let max_secs: u64 = 2;
    let max_clocks: u64 = max_secs * cpu_freq;

    let mut measureable_actions: Vec<Box<dyn MeasureableAction>> = Vec::new();
    measureable_actions.push(Box::new(FSReadRepetition{ file_bytes: Vec::new() }));
        
    for measureable_action in &mut measureable_actions {
        let mut rtd = RepetitionTestData {
            iterations: 0,
            total_clocks: 0,
            min_clocks: u64::MAX,
            max_clocks: 0
        };
        let mut running_iterations: u64 = 0;
        let mut running_clocks: u64 = 0;
        let mut byte_count;

        loop {
            let starting_stamp = clocks::read_cpu_timer();
            let results = (*measureable_action).action();
            let ending_stamp = clocks::read_cpu_timer();
            let diff_stamp = ending_stamp - starting_stamp;

            byte_count = results.byte_count;

            rtd.iterations += 1;
            rtd.total_clocks += diff_stamp;

            running_iterations += 1;
            running_clocks += diff_stamp;

            (*measureable_action).reset();

            if diff_stamp < rtd.min_clocks  {
                rtd.min_clocks = diff_stamp;
                running_clocks = 0;
                running_iterations = 0;
            } else if diff_stamp > rtd.max_clocks  {
                rtd.max_clocks = diff_stamp;
            }

            if running_iterations >= max_iterations && running_clocks >= max_clocks {
                break;
            }
        }

        let gigabyte_count = byte_count as f64 / 1024.0 / 1024.0 / 1024.0;
        let min_time = rtd.min_clocks as f64 / cpu_freq as f64;
        let min_bandwidth = gigabyte_count / min_time;
        let max_time = rtd.max_clocks as f64 / cpu_freq as f64;
        let max_bandwidth = gigabyte_count / max_time;
        let avg_clocks = rtd.total_clocks as f64 / rtd.iterations as f64;
        let avg_time = avg_clocks as f64 / cpu_freq as f64;
        let avg_bandwidth = gigabyte_count / avg_time;

        println!("--- {} ---", (*measureable_action).tag());
        println!("Min: {} ({:.6}) {:.6}gb/s", rtd.min_clocks, min_time, min_bandwidth);
        println!("Max: {} ({:.6}) {:.6}gb/s", rtd.max_clocks, max_time, max_bandwidth);
        println!("Avg: {} ({:.6}) {:.6}gb/s", avg_clocks, avg_time, avg_bandwidth);
    }
}
