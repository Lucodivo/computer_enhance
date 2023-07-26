use std::mem;
use core::arch::x86_64::_rdtsc;

#[inline(always)]
pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub fn measure_cpu_freq(measure_time_ms: u64) -> u64 {

    #[cfg(target_os = "windows")]
    fn get_os_timer_freq() -> u64 { unsafe {
        let mut freq = mem::zeroed();
        winapi::um::profileapi::QueryPerformanceFrequency(&mut freq);
        *freq.QuadPart() as u64
    }}

    #[cfg(target_os = "windows")]
    fn read_os_timer() -> u64 { unsafe {
        let mut freq = mem::zeroed();
        winapi::um::profileapi::QueryPerformanceCounter(&mut freq);
        *freq.QuadPart() as u64
    }}

    assert!(measure_time_ms > 0, "measure_time_ms must be greater than 0.");

    let os_clocks_per_sec = get_os_timer_freq();
    let os_wait_clocks = os_clocks_per_sec / 1000 * measure_time_ms;

    let mut os_timer_elapsed: u64 = 0;
    let mut os_timer_end: u64;
    let cpu_timer_start = read_cpu_timer();
    let os_timer_start = read_os_timer();
    while os_timer_elapsed < os_wait_clocks {
        os_timer_end = read_os_timer();
        os_timer_elapsed = os_timer_end - os_timer_start;
    }
    let cpu_timer_end = read_cpu_timer();
    let cpu_timer_elapsed = cpu_timer_end - cpu_timer_start;
    let cpu_clocks_per_second = (cpu_timer_elapsed * os_clocks_per_sec) / os_timer_elapsed;
    return cpu_clocks_per_second;
}

pub fn clocks_to_secs(clocks: u64, cpu_freq: u64) -> f64 { clocks as f64 / cpu_freq as f64 }
pub fn clocks_to_millisecs(clocks: u64, cpu_freq: u64) -> f64 { clocks_to_secs(clocks, cpu_freq) * 1000.0 }