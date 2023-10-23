use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::System::ProcessStatus::*;

static TEST_FILE: &str = "../../part_2/data/seed_123123/data_1000000_flex.json";

struct RepetitionTestData {
    iterations: u64,
    total_clocks: u64,
    total_page_faults: u64,
    min_clocks: u64,
    min_page_faults: u32,
    max_clocks: u64,
    max_page_faults: u32,
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
        Self {
            file_bytes: Vec::new(),
        }
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
    fn tag(&self) -> &'static str {
        "fs::read"
    }
}

struct FSFileRepetition {
    file: Option<std::fs::File>,
    file_bytes: Vec<u8>,
}

impl FSFileRepetition {
    fn new() -> Self {
        Self {
            file: None,
            file_bytes: Vec::new(),
        }
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
    fn tag(&self) -> &'static str {
        "fs::file"
    }
}

struct FSReadAsStringRepetition {
    file_as_string: String,
}

impl FSReadAsStringRepetition {
    fn new() -> Self {
        Self {
            file_as_string: String::new(),
        }
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
    fn tag(&self) -> &'static str {
        "fs::read_as_string"
    }
}

struct FSFileNoAllocationRepetition {
    file: Option<std::fs::File>,
    file_buffer: Vec<u8>,
}

impl FSFileNoAllocationRepetition {
    fn new() -> Self {
        let file_metadata =
            std::fs::metadata(TEST_FILE).expect("Couldn't get fs::metadata for file");
        Self {
            file: None,
            file_buffer: Vec::with_capacity(file_metadata.len() as usize),
        }
    }
}

impl MeasureableAction for FSFileNoAllocationRepetition {
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
    fn tag(&self) -> &'static str {
        "fs::read buffer no alloc"
    }
}

fn get_page_faults(process_handle: HANDLE) -> u32 {
    let mut memory_counters = PROCESS_MEMORY_COUNTERS{
        cb : 0,
        PageFaultCount : 0,
        PeakWorkingSetSize : 0,
        WorkingSetSize : 0,
        QuotaPeakPagedPoolUsage : 0,
        QuotaPagedPoolUsage : 0,
        QuotaPeakNonPagedPoolUsage : 0,
        QuotaNonPagedPoolUsage : 0,
        PagefileUsage : 0,
        PeakPagefileUsage : 0,
    };

    memory_counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    unsafe {
        GetProcessMemoryInfo(process_handle, &mut memory_counters, memory_counters.cb);
    }
    return memory_counters.PageFaultCount;
}

fn main() {
    let cpu_freq = clocks::measure_cpu_freq(1000);
    let process_id : u32;
    let process_handle : HANDLE;
    unsafe {
        process_id = GetCurrentProcessId();
        process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, windows_sys::Win32::Foundation::FALSE, process_id);
    }

    let max_iterations: u64 = 100;
    let max_secs: u64 = 5;
    let max_clocks: u64 = max_secs * cpu_freq;

    let mut measureable_actions: Vec<Box<dyn MeasureableAction>> = Vec::new();
    measureable_actions.push(Box::new(FSReadRepetition::new()));
    measureable_actions.push(Box::new(FSFileRepetition::new()));
    measureable_actions.push(Box::new(FSReadAsStringRepetition::new()));
    measureable_actions.push(Box::new(FSFileNoAllocationRepetition::new()));
    let mut repetition_test_data = Vec::<RepetitionTestData>::new();
    repetition_test_data.resize_with(measureable_actions.len(), || RepetitionTestData {
        iterations: 0,
        total_clocks: 0,
        total_page_faults: 0,
        min_clocks: u64::MAX,
        min_page_faults: u32::MAX,
        max_clocks: 0,
        max_page_faults: 0,
    });

    loop {
        // loop until user force quits
        let mut repetitions = measureable_actions
            .iter_mut()
            .zip(repetition_test_data.iter_mut());
        for (action, rtd) in &mut repetitions {
            println!("--- {} ---", (*action).tag());

            let mut running_iterations: u64 = 0;
            let mut running_clocks: u64 = 0;
            let mut byte_count;

            loop {
                let starting_page_faults = get_page_faults(process_handle);
                let starting_stamp = clocks::read_cpu_timer();
                let results = (*action).action();
                let ending_stamp = clocks::read_cpu_timer();
                let ending_page_faults = get_page_faults(process_handle);
                let diff_stamp = ending_stamp - starting_stamp;
                let diff_page_faults = ending_page_faults - starting_page_faults;
                byte_count = results.byte_count;

                rtd.iterations += 1;
                rtd.total_clocks += diff_stamp;
                rtd.total_page_faults += diff_page_faults as u64;

                running_iterations += 1;
                running_clocks += diff_stamp;

                (*action).reset();

                if diff_stamp < rtd.min_clocks {
                    rtd.min_clocks = diff_stamp;
                    rtd.min_page_faults = diff_page_faults;
                    running_clocks = 0;
                    running_iterations = 0;
                } else if diff_stamp > rtd.max_clocks {
                    rtd.max_clocks = diff_stamp;
                    rtd.max_page_faults = diff_page_faults;
                }

                let gigabyte_count = byte_count as f64 / 1024.0 / 1024.0 / 1024.0;
                let min_time = rtd.min_clocks as f64 / cpu_freq as f64;
                let min_bandwidth = gigabyte_count / min_time;
                if rtd.min_page_faults > 0 {
                    let min_kb_per_pagefault = ((byte_count as f64) / 1024.0) / rtd.min_page_faults as f64; 
                    print!(
                        "\rMin: {} clocks ({:.6}ms) {:.6}gb/s PF: {:.4} ({:.6}k/fault)",
                        rtd.min_clocks, min_time, min_bandwidth, rtd.min_page_faults, min_kb_per_pagefault
                        );
                } else {
                    print!(
                        "\rMin: {} clocks ({:.6}ms) {:.6}gb/s                                  ",
                        rtd.min_clocks, min_time, min_bandwidth,
                        );
                }

                if running_iterations >= max_iterations && running_clocks >= max_clocks {
                    println!();
                    break;
                }
            }

            let gigabyte_count = byte_count as f64 / 1024.0 / 1024.0 / 1024.0;
            let max_time = rtd.max_clocks as f64 / cpu_freq as f64;
            let max_bandwidth = gigabyte_count / max_time;
            let avg_clocks = (rtd.total_clocks as f64 / rtd.iterations as f64) as u64;
            let avg_time = avg_clocks as f64 / cpu_freq as f64;
            let avg_bandwidth = gigabyte_count / avg_time;

            if rtd.max_page_faults > 0 {
                let max_kb_per_pagefault = ((byte_count as f64) / 1024.0) / rtd.max_page_faults as f64; 
                println!(
                    "Max: {} clocks ({:.6}ms) {:.6}gb/s PF: {:.4} ({:.6}k/fault)",
                    rtd.max_clocks, max_time, max_bandwidth, rtd.max_page_faults, max_kb_per_pagefault
                    );
            } else {
                println!(
                    "Max: {} clocks ({:.6}ms) {:.6}gb/s                                  ",
                    rtd.max_clocks, max_time, max_bandwidth,
                    );
            }
            
            if rtd.total_page_faults > 0 {
                let avg_page_faults = rtd.total_page_faults as f64 / rtd.iterations as f64;
                let avg_kb_per_pagefault = ((byte_count as f64) / 1024.0) / avg_page_faults as f64; 
                println!(
                    "Avg: {} clocks ({:.6}ms) {:.6}gb/s PF: {:.4} ({:.6}k/fault)",
                    avg_clocks, avg_time, avg_bandwidth, avg_page_faults, avg_kb_per_pagefault
                    );
            } else {
                println!(
                    "Avg: {} clocks ({:.6}ms) {:.6}gb/s                                  ",
                    avg_clocks, avg_time, avg_bandwidth
                    );
            }
            
        }
    }
}
