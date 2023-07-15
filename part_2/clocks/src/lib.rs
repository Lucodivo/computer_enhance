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

pub fn testing() {
    let os_freq = get_os_timer_freq();
    println!("OS Timer Frequency: {}", os_freq);

    let os_start = read_os_timer();
    let mut os_end: u64 = 0;
    let mut os_elapsed: u64 = 0;
    while os_elapsed < os_freq {
        os_end = read_os_timer();
        os_elapsed = os_end - os_start;
    }

    println!("OS Timer: {} -> {} = {} elapsed", os_start, os_end, os_elapsed);
    println!("OS Seconds: {:.4}", (os_elapsed as f64) / (os_freq as f64));
}