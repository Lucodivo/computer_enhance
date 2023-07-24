// NOTE: This profiler is absolutely not multi-threaded safe
pub use utils::Defer;
pub use once_cell::sync::Lazy;

use clocks::{measure_cpu_freq, clocks_to_millisecs, read_cpu_timer};
use utils::{ printable_freq, printable_large_num};

#[derive(Clone, Copy)]
struct Profile { 
    tag: &'static str,
    duration: u64,
    open_stamp: u64,
    open_count: u64,
    closed_count: u64,
    child_duration: u64,
    current_scope: usize
}

// NOTE: This is only used for the profiler macros. DO NOT use this for anything else. 
#[allow(non_snake_case)]
pub fn __PROFILER__COUNTER__() -> usize {
    static mut COUNTER: usize = 0;
    unsafe {
        // Counting starting at 1 is intended and not to be changed
        // 0 is reserved for a default value
        COUNTER += 1;
        COUNTER
    }
}

const PROFILE_CAPACITY: usize = 4096;
pub struct Profiler { 
    creation_stamp: u64, 
    cpu_freq: u64, 
    profiles: [Profile; PROFILE_CAPACITY],
    current_scope: usize
}
pub static mut PROFILER: Profiler = Profiler{ 
    creation_stamp: 0, 
    cpu_freq: 0, 
    profiles: [Profile{ 
        tag: "_", 
        duration: 0, 
        open_stamp: 0, 
        open_count: 0, 
        closed_count: 0, 
        child_duration: 0, 
        current_scope: 0 
    }; PROFILE_CAPACITY],
    current_scope: 0
};
impl Profiler {
    fn init(&mut self) {
        self.cpu_freq = measure_cpu_freq(100);
        self.creation_stamp = read_cpu_timer();
    }
    fn deinit(&self) {
        let now = read_cpu_timer();
        let total_clocks = now - self.creation_stamp;
        let total_millisecs = clocks_to_millisecs(total_clocks, self.cpu_freq);

        println!("\n====== Profiler Results *START* =======\n");
        
        let clocks_column_title = format!("Clocks @ {}", printable_freq(self.cpu_freq));
        println!("{:<40}{:<25}{:<10}{}\n", "Tag (Invocations)", clocks_column_title, "Percent", "w/ Children");

        let mut index = 1;
        while self.profiles[index].open_count > 0 {
            let profile = &self.profiles[index];
            assert!(profile.open_count == profile.closed_count, "Profile, with tag {}, was not registered/unregistered an equal amount of times.", profile.tag);
            let duration_minus_children = profile.duration - profile.child_duration;
            let percent_minus_children = duration_minus_children as f64 / total_clocks as f64 * 100.0;
            let percent_with_children = profile.duration as f64 / total_clocks as f64 * 100.0;
            let title_str = format!("{} ({}):", profile.tag, printable_large_num(profile.closed_count));
            println!("{:<40}{:<25}{:<10.2}{:.2}", title_str, printable_large_num(duration_minus_children), percent_minus_children, percent_with_children);
            index += 1;
        }
        assert!(self.current_scope == 0, "A profiler was registered without being unregistered.");

        let time_str = format!("{} ({:.3}ms)", printable_large_num(total_clocks), total_millisecs);
        println!("\n{:<40}{:<25}", "Total:", time_str);

        println!("\n====== Profiler Results *END* =======\n");
    }

    pub fn register(&mut self, tag: &'static str, index: usize) {

        let profile = &mut self.profiles[index];

        if profile.open_count == profile.closed_count {
            profile.open_count += 1;
            profile.tag = tag;
            profile.current_scope = self.current_scope;
            self.current_scope = index;
            profile.open_stamp = read_cpu_timer();
        } else { profile.open_count += 1; }
    }
    pub fn unregister(&mut self) {

        let index = self.current_scope;
        let profile = &mut self.profiles[index];
        profile.closed_count += 1;

        if profile.open_count == profile.closed_count {
            let duration = read_cpu_timer() - profile.open_stamp;
            profile.open_stamp = 0;
            profile.duration += duration;
            self.current_scope = profile.current_scope;
            profile.current_scope = 0;
            self.profiles[self.current_scope].child_duration += duration;
        }
    }

}

/*
Calling this macro twice in the same function will *NOT* compile.
*/
#[macro_export]
macro_rules! time_function {
    // `()` indicates that the macro takes no argument.
    () => {
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
        let name = type_name_of(f);
        let func_name_start = &name[..name.len() - 3].rfind(':').expect("Failed to find function name.") + 1;
        let func_name_end = name.len() - 3; // remove the trailing "::f"
        let func_name = &name[func_name_start..func_name_end];

        unsafe {
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register(func_name, *PROFILE_INDEX); 
        }

        let _defer_unregister = Defer::new( || {
            unsafe { PROFILER.unregister(); }
        });
    }
}

#[macro_export]
macro_rules! time_block {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        unsafe { 
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register($msg, *PROFILE_INDEX);
        }
        let _defer_unregister = Defer::new(|| {
            unsafe { PROFILER.unregister(); }
        });
    }
}

#[macro_export]
macro_rules! time_assignment_rhs  {
    ($x:stmt) => {
        let stmt = stringify!($x);
        let rhs = if let Some(equal_index) = stmt.find('=') {
            stmt[equal_index + 1..stmt.len()-1].trim_start()
        } else { &stmt[..stmt.len()-1] };
        unsafe { 
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register(rhs, *PROFILE_INDEX); 
        }
        $x;
        unsafe { PROFILER.unregister(); }
    }
}

#[macro_export]
macro_rules! time_assignment  {
    ($x:stmt) => {
        let stmt = stringify!($x);
        let lhs = if let Some(equal_index) = stmt.find('=') {
            if let Some(variable_name_index) = stmt[0..equal_index].trim_end().rfind(' ') {
                stmt[variable_name_index + 1..equal_index].trim() // simplify??
            } else { &stmt }
        } else { &stmt };
        unsafe {        
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() }); 
            PROFILER.register(lhs, *PROFILE_INDEX); 
        }
        $x;
        unsafe { PROFILER.unregister(); }
    }
}

#[macro_export]
macro_rules! time_assignments {
    ($($x:stmt);* $(;)?) => {
        $(
            time_assignment!($x);
        )*
    };
}

#[macro_export]
macro_rules! time_open {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        unsafe {            
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() }); 
            PROFILER.register($msg, *PROFILE_INDEX); 
        }
    }
}

#[macro_export]
macro_rules! time_close {
    // `()` indicates that the macro takes no argument.
    () => {
        unsafe { PROFILER.unregister(); }
    }
}

pub fn profiler_setup() {
    unsafe { PROFILER.init(); }
    time_function!();
}

pub fn profiler_teardown() {
    // NOTE: profiling teardown with PROFILER does not really work because the profiler will be closed on exit.
    unsafe { PROFILER.deinit(); }
}