// NOTE: This profiler is absolutely not multi-threaded safe
// NOTE: The profiler is aggressively removed from non-profile feature builds to ensure
//      that it does not affect the program in any way.

pub use utils::Defer;

#[cfg(feature = "profile")]
pub use once_cell::sync::Lazy;

#[cfg(feature = "profile")]
pub use std::mem::drop;

#[cfg(feature = "profile")]
use clocks::{measure_cpu_freq, clocks_to_millisecs, read_cpu_timer};

#[cfg(feature = "profile")]
use utils::{ printable_freq, printable_large_num};

#[cfg(feature = "profile")]
const PROFILE_CAPACITY: usize = 4096;
#[cfg(feature = "profile")]
static EMPTY_TAG: &str = "";

// NOTE: This is only used for the profiler macros. DO NOT use this for anything else. 
#[cfg(feature = "profile")]
#[allow(non_snake_case)]
pub fn __GLOBAL_PROFILER__COUNTER__() -> usize {
    static mut COUNTER: usize = 0;
    unsafe {
        // Counting starting at 1 is intended and not to be changed
        // 0 is reserved for a default value
        COUNTER += 1;
        COUNTER
    }
}

#[cfg(feature = "profile")]
#[derive(Clone, Copy, Default)]
pub struct ProfileAnchor {
    tag: &'static str,
    elapsed_exclusive: u64,
    elapsed_inclusive: u64,
    invocations: u64,
}

#[cfg(feature = "profile")]
pub struct Profiler {
    profiles: [ProfileAnchor; PROFILE_CAPACITY],
    creation_stamp: u64,
    teardown_start_stamp: u64
}
#[cfg(feature = "profile")]
pub static mut GLOBAL_PROFILER: Profiler = Profiler{ 
    profiles: [ProfileAnchor{ tag: EMPTY_TAG, elapsed_exclusive: 0, elapsed_inclusive: 0, invocations: 0 }; PROFILE_CAPACITY],
    creation_stamp: 0,
    teardown_start_stamp: 0
};
#[cfg(feature = "profile")]
pub static mut GLOBAL_PROFILER_SCOPE: usize = 0;

#[cfg(feature = "profile")]
pub struct ProfileBlock {
    tag: &'static str,
    creation_stamp: u64,
    old_elapsed_inclusive: u64,
    parent_index: usize,
    profile_index: usize
}

#[cfg(feature = "profile")]
impl ProfileBlock {
    pub fn new(tag: &'static str, profile_index: usize) -> Self {
        let parent_index;
        let old_elapsed_inclusive;
        unsafe {
            parent_index = GLOBAL_PROFILER_SCOPE;
            GLOBAL_PROFILER_SCOPE = profile_index;

            old_elapsed_inclusive = GLOBAL_PROFILER.profiles[profile_index].elapsed_inclusive;
        };
        ProfileBlock {
            tag,
            creation_stamp: read_cpu_timer(),
            old_elapsed_inclusive,
            parent_index,
            profile_index
        }
    }
}

#[cfg(feature = "profile")]
impl Drop for ProfileBlock {
    fn drop(&mut self) {
        let elapsed: u64 = read_cpu_timer() - self.creation_stamp;
        unsafe {
            GLOBAL_PROFILER_SCOPE = self.parent_index;

            let parent_profile = &mut GLOBAL_PROFILER.profiles[self.parent_index];
            let profile = &mut GLOBAL_PROFILER.profiles[self.profile_index];

            parent_profile.elapsed_exclusive = parent_profile.elapsed_exclusive.wrapping_sub(elapsed);
            profile.elapsed_exclusive = profile.elapsed_exclusive.wrapping_add(elapsed);
            profile.elapsed_inclusive = self.old_elapsed_inclusive + elapsed;
            profile.invocations += 1;
            profile.tag = self.tag;
        }
    }
}

#[cfg(feature = "profile")]
impl Profiler {
    pub fn init(&mut self) {
        unsafe { 
            GLOBAL_PROFILER.creation_stamp = read_cpu_timer(); 
            GLOBAL_PROFILER.profiles[0].tag = "Application";
        }
    }

    pub fn print_and_deinit(&mut self) {
        if self.teardown_start_stamp > 0 {
            ProfileBlock{ tag: "app teardown", creation_stamp: self.teardown_start_stamp, old_elapsed_inclusive: 0, parent_index: 0, profile_index: __GLOBAL_PROFILER__COUNTER__() };
        }
        let end_stamp = read_cpu_timer();
        let cpu_freq = measure_cpu_freq(100);

        let total_clocks = end_stamp - self.creation_stamp;
        let total_millisecs = clocks_to_millisecs(total_clocks, cpu_freq);

        // manually "drop" application-wide profile anchor
        let profiler_profile = &mut self.profiles[0];
        profiler_profile.elapsed_inclusive = total_clocks;
        profiler_profile.elapsed_exclusive = profiler_profile.elapsed_exclusive.wrapping_add(total_clocks);
        let clocks_profiled = profiler_profile.elapsed_inclusive - profiler_profile.elapsed_exclusive;

        print!("\n====== Profiler Results *START* =======");

        let clocks_column_title = format!("Clocks @ {}", printable_freq(cpu_freq));
        println!("\n{:<40}{:<25}{:<10}{}\n", "Tag (Invocations)", clocks_column_title, "Percent", "w/ Children");

        for profile in self.profiles[1..].iter() {
            if profile.tag != EMPTY_TAG {
                    let percent_exclusive = profile.elapsed_exclusive as f64 / total_clocks as f64 * 100.0;
                    let percent_inclusive = profile.elapsed_inclusive as f64 / total_clocks as f64 * 100.0;
                    let title_str = format!("{} ({}):", profile.tag, printable_large_num(profile.invocations));
                    println!("{:<40}{:<25}{:<10.2}{:.2}", title_str, printable_large_num(profile.elapsed_exclusive), percent_exclusive, percent_inclusive);
            } else {
                break;
            }
        }
        
        let percent_profiled = clocks_profiled as f64 / total_clocks as f64 * 100.0;
        println!("\n{:<40}{:<25}{:<10.2}", "Total profiled:", printable_large_num(clocks_profiled as u64), percent_profiled);
        
        let total_clocks_str = format!("{} ({:.3}ms)", printable_large_num(total_clocks), total_millisecs);
        println!("\n{:<40}{:<25}\n", "Total runtime clocks:", total_clocks_str);

        println!("====== Profiler Results *END* =======\n");
    }

    pub fn time_teardown(&mut self) { self.teardown_start_stamp = read_cpu_timer(); }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_block {
    // `()` indicates that the macro takes no argument.
    ( $tag:expr ) => {
        let __profiler_index: usize;
        unsafe {
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { profiler::__GLOBAL_PROFILER__COUNTER__() });
            __profiler_index = *PROFILE_INDEX;
        };
        let __profile_block = profiler::ProfileBlock::new($tag, __profiler_index);
    }
}

/*
Calling this macro twice in the same function will *NOT* compile.
*/
#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_function {
    // `()` indicates that the macro takes no argument.
    () => {
        let __profiler_tag: &'static str;
        unsafe {
            fn f() {}
            #[inline(always)]
            fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| { 
                    let name = type_name_of(f);
                    let func_name_start = &name[..name.len() - 3].rfind(':').expect("Failed to find function name.") + 1;
                    let func_name_end = name.len() - 3; // remove the trailing "::f"
                    &name[func_name_start..func_name_end]
            });
            __profiler_tag = *PROFILE_TAG;
        };
        profiler::time_block!(__profiler_tag);
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! profiler_setup_defer_teardown {
    () => {
        // This is kept even if not profiling, as it does not add any overhead during the running of the application
        unsafe { profiler::GLOBAL_PROFILER.init(); } 
        
        let _defer_unregister = Defer::new(|| { unsafe { profiler::GLOBAL_PROFILER.print_and_deinit(); } });
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_section {
    ( $tag:expr, $($x:stmt);* $(;)? ) => {
        let __profiler_index: usize;
        unsafe {
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { profiler::__GLOBAL_PROFILER__COUNTER__() });
            __profiler_index = *PROFILE_INDEX;
        };
        let __profile_section = profiler::ProfileBlock::new($tag, __profiler_index);
        $($x;)*
        drop(__profile_section);
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_assignment_rhs  {
    ($x:stmt) => {
        let __profiler_tag: &'static str;
        let __profiler_index: usize;
        unsafe { 
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| {
                let stmt = stringify!($x);
                if let Some(equal_index) = stmt.find('=') {
                    stmt[equal_index + 1..stmt.len()-1].trim_start()
                } else { &stmt[..stmt.len()-1] };
            });
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { profiler::__GLOBAL_PROFILER__COUNTER__() });
            __profiler_tag = *PROFILE_TAG;
            __profiler_index = *PROFILE_INDEX;
        }
        let __profile_section = profiler::ProfileBlock::new(__profiler_tag, __profiler_index);
        $x;
        drop(__profile_section);
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_assignment  {
    ($x:stmt) => {
        let __profiler_tag: &'static str;
        let __profiler_index: usize;
        unsafe {        
            static PROFILE_TAG: Lazy<&'static str> = Lazy::new(|| {
                let stmt = stringify!($x);
                if let Some(equal_index) = stmt.find('=') {
                    if let Some(variable_name_index) = stmt[0..equal_index].trim_end().rfind(' ') {
                        stmt[variable_name_index + 1..equal_index].trim() // simplify??
                    } else { &stmt }
                } else { &stmt }
            });
            static PROFILE_INDEX: Lazy<usize> = Lazy::new(|| { profiler::__GLOBAL_PROFILER__COUNTER__() });
            __profiler_tag = *PROFILE_TAG;
            __profiler_index = *PROFILE_INDEX;
        }
        let __profile_section = profiler::ProfileBlock::new(__profiler_tag, __profiler_index);
        $x;
        drop(__profile_section);
    }
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_assignments {
    ($($x:stmt);* $(;)?) => { $( profiler::time_assignment!($x); )* };
}

#[cfg(feature = "profile")]
#[macro_export]
macro_rules! time_teardown { () => { unsafe{ profiler::GLOBAL_PROFILER.time_teardown(); } } }

/* 
    Alternative macros for when profiling is *NOT* enabled.
 */
#[cfg(not(feature = "profile"))]
pub use utils::{ printable_freq, printable_large_num};
#[cfg(not(feature = "profile"))]
pub use clocks::{measure_cpu_freq, clocks_to_millisecs, read_cpu_timer};

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_block { ( $msg:expr ) => {} }

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_function { () => {} }

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! profiler_setup_defer_teardown {
    () => {
        // Note: I still want to get some information on the total runtime of the program.
        let start_stamp = read_cpu_timer();
        
        let _defer_unregister = Defer::new(|| { 
            let end_stamp = read_cpu_timer();
            let cpu_freq = measure_cpu_freq(100);
            let total_clocks = end_stamp - start_stamp;
            let total_millisecs = clocks_to_millisecs(total_clocks, cpu_freq);
            let total_clocks_str = format!("{} ({:.3}ms)", printable_large_num(total_clocks), total_millisecs);
            println!("\nProfiler turned off.");
            println!("Clocks @ {}", printable_freq(cpu_freq));
            println!("Total runtime clocks: {}\n", total_clocks_str);
         });
    }
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_section {
    ( $tag:expr, $($x:stmt);* $(;)? ) => { $($x;)* }
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_assignment_rhs  {
    ($x:stmt) => { $x; }
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_assignment  {
    ($x:stmt) => { $x; }
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_assignments {
    ($($x:stmt);* $(;)?) => { $($x)* };
}

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! time_teardown { () => {} }