// NOTE: This profiler is absolutely not multi-threaded safe
pub use utils::Defer;
pub use once_cell::sync::Lazy;

use clocks::{measure_cpu_freq, clocks_to_millisecs, read_cpu_timer};
use utils::{ printable_freq, printable_large_num};

#[derive(Clone, Copy)]
struct Profile { 
    tag: &'static str,
    duration: u64,
    open_count: u64,
    closed_count: u64,
    child_duration: i64
}

#[derive(Clone, Copy)]
struct ProfilerStamp {
    tag: &'static str,
    index: usize,
    stamp: u64
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
    cpu_freq: u64, 
    profiles: [Profile; PROFILE_CAPACITY],
    open_stamps: [ProfilerStamp; PROFILE_CAPACITY],
    open_stamps_count: usize,
    time_teardown: bool
}
pub static mut PROFILER: Profiler = Profiler{ 
    cpu_freq: 0,
    profiles: [Profile{ 
        tag: "_", 
        duration: 0,
        open_count: 0, 
        closed_count: 0, 
        child_duration: 0 
    }; PROFILE_CAPACITY],
    open_stamps: [ProfilerStamp{ 
        tag: "_", 
        index: 0, 
        stamp: 0 
    }; PROFILE_CAPACITY],
    open_stamps_count: 0,
    time_teardown: false
};

impl Profiler {
    pub fn init(&mut self) {
        self.cpu_freq = measure_cpu_freq(100);
        self.register("Profiler", 0);
    }
    pub fn deinit(&mut self) {
        if self.time_teardown { self.unregister() }

        // manually deregister the profiler profile
        let now = read_cpu_timer();
        self.open_stamps_count -= 1;
        assert_eq!(self.open_stamps_count, 0, "A profile was registered without being unregistered or vice versa.");
        let profiler_stamp = &self.open_stamps[0];
        let profiler_profile = &mut self.profiles[0];
        profiler_profile.tag = profiler_stamp.tag;
        profiler_profile.duration += now - profiler_stamp.stamp;
        profiler_profile.closed_count += 1;
        let profiler_profile = &self.profiles[0];

        let total_clocks = profiler_profile.duration;
        let total_millisecs = clocks_to_millisecs(profiler_profile.duration, self.cpu_freq);

        println!("\n====== Profiler Results *START* =======\n");
        
        let clocks_column_title = format!("Clocks @ {}", printable_freq(self.cpu_freq));
        println!("{:<40}{:<25}{:<10}{}\n", "Tag (Invocations)", clocks_column_title, "Percent", "w/ Children");

        let mut index = 1;
        while self.profiles[index].open_count > 0 {
            let profile = &self.profiles[index];
            assert!(profile.open_count == profile.closed_count, "Profile, with tag {}, was not registered/unregistered an equal amount of times.", profile.tag);
            assert!(profile.child_duration >= 0, "Profile, with tag {}, has a negative child duration.", profile.tag);
            let duration_minus_children = profile.duration - profile.child_duration as u64;
            let percent_minus_children = duration_minus_children as f64 / total_clocks as f64 * 100.0;
            let percent_with_children = profile.duration as f64 / total_clocks as f64 * 100.0;
            let title_str = format!("{} ({}):", profile.tag, printable_large_num(profile.closed_count));
            println!("{:<40}{:<25}{:<10.2}{:.2}", title_str, printable_large_num(duration_minus_children), percent_minus_children, percent_with_children);
            
            index += 1;
        }
        
        let total_percent_profiled = profiler_profile.child_duration as f64 / total_clocks as f64 * 100.0;
        println!("\n{:<40}{:<25}{:<10.2}", "Total profiled:", printable_large_num(profiler_profile.child_duration as u64), total_percent_profiled);
        
        let total_clocks_str = format!("{} ({:.3}ms)", printable_large_num(total_clocks), total_millisecs);
        println!("\n{:<40}{:<25}", "Total runtime clocks:", total_clocks_str);
        
        println!("\n====== Profiler Results *END* =======\n");
    }

    pub fn register(&mut self, tag: &'static str, index: usize) {

        let profile_stamp = &mut self.open_stamps[self.open_stamps_count];
        self.open_stamps_count += 1;

        self.profiles[index].open_count += 1;
        
        profile_stamp.tag = tag;
        profile_stamp.index = index;
        profile_stamp.stamp = read_cpu_timer();
    }
    
    pub fn unregister(&mut self) {

        let now = read_cpu_timer();

        self.open_stamps_count -= 1;
        let profile_stamp = &self.open_stamps[self.open_stamps_count];

        let duration = now - profile_stamp.stamp;

        let profile = &mut self.profiles[profile_stamp.index];
        profile.closed_count += 1;
        if profile.open_count == profile.closed_count {
            profile.tag = profile_stamp.tag;
            profile.duration += duration;
        } else {
            profile.child_duration -= duration as i64;
        }

        let parent_stamp = &self.open_stamps[self.open_stamps_count - 1];
        if parent_stamp.index != profile_stamp.index {
            self.profiles[parent_stamp.index].child_duration += duration as i64;
        }
    }

    pub fn time_app_teardown(&mut self) {
        self.time_teardown = true;
        static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() }); 
        self.register("application teardown", *PROFILE_INDEX); 
    }

}

/*
Calling this macro twice in the same function will *NOT* compile.
*/
#[macro_export]
macro_rules! time_function {
    // `()` indicates that the macro takes no argument.
    () => {
        #[cfg(feature = "profile")]
        unsafe {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| { 
                let name = type_name_of(f);
                let func_name_start = &name[..name.len() - 3].rfind(':').expect("Failed to find function name.") + 1;
                let func_name_end = name.len() - 3; // remove the trailing "::f"
                &name[func_name_start..func_name_end]
            });
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register(*PROFILE_TAG, *PROFILE_INDEX); 
        }

        #[cfg(feature = "profile")]
        let _defer_unregister = Defer::new( || {
            unsafe { PROFILER.unregister(); }
        });
    }
}

#[macro_export]
macro_rules! time_block {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        #[cfg(feature = "profile")]
        unsafe { 
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register($msg, *PROFILE_INDEX);
        }
        
        #[cfg(feature = "profile")]
        let _defer_unregister = Defer::new(|| {
            unsafe { PROFILER.unregister(); }
        });
    }
}

#[macro_export]
macro_rules! time_assignment_rhs  {
    ($x:stmt) => {
        #[cfg(feature = "profile")]
        unsafe { 
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| {
                let stmt = stringify!($x);
                if let Some(equal_index) = stmt.find('=') {
                    stmt[equal_index + 1..stmt.len()-1].trim_start()
                } else { &stmt[..stmt.len()-1] };
            });
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() });
            PROFILER.register(*PROFILE_TAG, *PROFILE_INDEX); 
        }
        $x;
        #[cfg(feature = "profile")]
        unsafe { PROFILER.unregister(); }
    }
}

#[macro_export]
macro_rules! time_assignment  {
    ($x:stmt) => {
        #[cfg(feature = "profile")]
        unsafe {        
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| {
                let stmt = stringify!($x);
                if let Some(equal_index) = stmt.find('=') {
                    if let Some(variable_name_index) = stmt[0..equal_index].trim_end().rfind(' ') {
                        stmt[variable_name_index + 1..equal_index].trim() // simplify??
                    } else { &stmt }
                } else { &stmt }
            });
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() }); 
            PROFILER.register(*PROFILE_TAG, *PROFILE_INDEX); 
        }
        $x;
        #[cfg(feature = "profile")]
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
    ( $msg:expr ) => {
        #[cfg(feature = "profile")]
        unsafe {            
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { __PROFILER__COUNTER__() }); 
            PROFILER.register($msg, *PROFILE_INDEX); 
        }
    }
}

#[macro_export]
macro_rules! time_close {
    () => {
        #[cfg(feature = "profile")]
        unsafe { PROFILER.unregister(); }
    }
}

#[macro_export]
macro_rules! profiler_setup_defer_teardown {
    () => {
        // This is kept even if not profiling, as it does not add any overhead during the running of the application
        unsafe { PROFILER.init(); } 
            
        let _defer_unregister = Defer::new(|| { unsafe { PROFILER.deinit(); } });
    }
}

#[macro_export]
macro_rules! time_app_teardown {
    () => {
        #[cfg(feature = "profile")]
        unsafe { PROFILER.time_app_teardown(); }
    }
}