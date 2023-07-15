use std::mem;
use core::arch::x86_64::_rdtsc;

#[cfg(target_os = "windows")]
fn get_os_timer_freq() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        winapi::um::profileapi::QueryPerformanceFrequency(&mut freq);
        *freq.QuadPart() as u64
    }
}

#[cfg(target_os = "windows")]
fn read_os_timer() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        winapi::um::profileapi::QueryPerformanceCounter(&mut freq);
        *freq.QuadPart() as u64
    }
}

fn read_cpu_timer() -> u64 {
    // RDTSC — Read Time-Stamp Counter
    unsafe { _rdtsc() }
}

fn printable_freq(freq: u64) -> String {
    match freq {
        0..=999 => format!("{} Hz", freq),
        1_000..=999_999 => format!("{:.3} KHz", (freq as f64) / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.2} MHz", (freq as f64) / 1_000_000.0),
        1_000_000_000..=999_999_999_999 => format!("{:.1} GHz", (freq as f64) / 1_000_000_000.0),
        _ => format!("{:.1} THz", (freq as f64) / 1_000_000_000_000.0)
    }
}

fn printable_large_num(number: u64) -> String {
    number.to_string()
        .as_bytes()
        .rchunks(3).rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

pub fn testing(measure_time_ms: u64) {
    assert!(measure_time_ms > 0, "measure_time_ms must be greater than 0.");

    let os_timer_freq = get_os_timer_freq();
    let os_wait_clocks = os_timer_freq / 1000 * measure_time_ms;

    let mut os_timer_elapsed: u64 = 0;
    let mut os_timer_end: u64 = 0;
    let cpu_timer_start = read_cpu_timer();
    let os_timer_start = read_os_timer();
    while os_timer_elapsed < os_wait_clocks {
        os_timer_end = read_os_timer();
        os_timer_elapsed = os_timer_end - os_timer_start;
    }
    let cpu_timer_end = read_cpu_timer();
    let cpu_timer_elapsed = cpu_timer_end - cpu_timer_start;
    let cpu_clocks_per_os_clock = cpu_timer_elapsed / os_timer_elapsed;
    let cpu_timer_freq = cpu_clocks_per_os_clock * os_timer_freq;

    println!("OS Timer Frequency: {}", printable_freq(os_timer_freq));
    println!("OS Timer: {} -> {} = {} clocks elapsed", printable_large_num(os_timer_start), printable_large_num(os_timer_end), printable_large_num(os_timer_elapsed));
    println!("OS Seconds Elapsed: {:.4}", (os_timer_elapsed as f64) / (os_timer_freq as f64));
    println!("CPU Timer: {} -> {} = {} clocks elapsed", printable_large_num(cpu_timer_start), printable_large_num(cpu_timer_end), printable_large_num(cpu_timer_elapsed));
    println!("CPU Timer Frequency: {}", printable_freq(cpu_timer_freq));
}