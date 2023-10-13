static TEST_FILE : &str = "../data/seed_123123/data_1000000_flex.json";

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

impl FSReadRepetition {
    fn new() -> Self {
        Self { file_bytes: Vec::new(), }
    }
}

impl MeasureableAction for FSReadRepetition {
    fn action(&mut self) -> MeasureableActionResults {
        self.file_bytes = std::fs::read(TEST_FILE).expect("Couldn't fs::read file");
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

struct FSFileRepetition {
    file: Option<std::fs::File>,
    file_bytes: Vec<u8>,
}

impl FSFileRepetition {
    fn new() -> Self {
        Self { file: None, file_bytes: Vec::new(), }
    }
}

impl MeasureableAction for FSFileRepetition {
    fn action(&mut self) -> MeasureableActionResults {
        use std::io::Read;
        self.file = Some(std::fs::File::open(TEST_FILE).expect("Couldn't fs::read file"));
        if let Some(file) = &mut self.file {
            let _ = file.read_to_end(&mut self.file_bytes).unwrap();
        }
        MeasureableActionResults {
            byte_count: self.file_bytes.len() as u64,
        }
    }
    fn reset(&mut self) {
        self.file = None;
        // this deallocates anything previously held on by file_bytes
        // .clear() would not deallocate
        self.file_bytes = Vec::new();
    }
    fn tag(&self) -> &'static str{ "fs::file" }
}

struct FSReadAsStringRepetition {
    file_as_string: String,
}

impl FSReadAsStringRepetition {
    fn new() -> Self {
        Self { file_as_string: String::new(), }
    }
}

impl MeasureableAction for FSReadAsStringRepetition {
    fn action(&mut self) -> MeasureableActionResults {
        self.file_as_string = std::fs::read_to_string(TEST_FILE).expect("Couldn't fs::read file");
        MeasureableActionResults {
            byte_count: self.file_as_string.len() as u64,
        }
    }
    fn reset(&mut self) {
        // this deallocates anything previously held on by file_bytes
        // .clear() would not deallocate
        self.file_as_string = String::new();
    }
    fn tag(&self) -> &'static str{ "fs::read_as_string" }
}

struct FSReadBufferNoAllocRepetition {
    file: Option<std::fs::File>,
    file_buffer: Vec<u8>,
}

impl FSReadBufferNoAllocRepetition {
    fn new() -> Self {
        let file_metadata = std::fs::metadata(TEST_FILE).expect("Couldn't get fs::metadata for file");
        Self { file: None, file_buffer: Vec::with_capacity(file_metadata.len() as usize), }
    }
}

impl MeasureableAction for FSReadBufferNoAllocRepetition {
    fn action(&mut self) -> MeasureableActionResults {
        use std::io::Read;
        self.file = Some(std::fs::File::open(TEST_FILE).expect("Couldn't fs::read file"));
        let mut bytes_read = 0u64;
        if let Some(file) = &mut self.file {
            bytes_read = file.read_to_end(&mut self.file_buffer).unwrap() as u64;
        }
        MeasureableActionResults {
            byte_count: bytes_read,
        }
    }
    fn reset(&mut self) {
        self.file = None;
        self.file_buffer.clear();
    }
    fn tag(&self) -> &'static str{ "fs::read buffer no alloc" }
}

fn main() {
    let cpu_freq = clocks::measure_cpu_freq(1000);

    let max_iterations: u64 = 10;
    let max_secs: u64 = 2;
    let max_clocks: u64 = max_secs * cpu_freq;

    let mut measureable_actions: Vec<Box<dyn MeasureableAction>> = Vec::new();
    measureable_actions.push(Box::new(FSReadRepetition::new()));
    measureable_actions.push(Box::new(FSFileRepetition::new()));
    measureable_actions.push(Box::new(FSReadAsStringRepetition::new()));
    measureable_actions.push(Box::new(FSReadBufferNoAllocRepetition::new()));
        
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
