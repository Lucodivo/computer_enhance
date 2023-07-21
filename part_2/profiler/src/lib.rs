use clocks::{measure_cpu_freq, read_cpu_timer, clocks_to_secs};
pub use utils::Defer;

enum ProfilerNode { 
    Open { stamp: u64, name: &'static str }, 
    Close { stamp: u64, name: &'static str }
}
pub struct Profiler { creation_stamp: u64, cpu_freq: u64, nodes: Vec<ProfilerNode> }
impl Profiler {
    fn init(&mut self) {
        self.cpu_freq = measure_cpu_freq(100);
        self.creation_stamp = read_cpu_timer();
    }
    fn deinit(&self) {
        let now = read_cpu_timer();
        let total_clocks = now - self.creation_stamp;
        let total_secs = clocks_to_secs(total_clocks, self.cpu_freq);        

        let mut open_nodes: Vec<&ProfilerNode> = Vec::new();
        for node in &self.nodes {
            match node {
                ProfilerNode::Open{ .. } => {
                    open_nodes.push(&node);
                },
                ProfilerNode::Close{ stamp: close_stamp, name: close_name } => {
                    if let ProfilerNode::Open{ stamp: open_stamp, name: open_name } = open_nodes.pop().expect("A profiler was unregistered without being registered.") {
                        assert!(open_name == close_name, "Unmatched names for register and unregister found.");
                        let delta_clocks = close_stamp - open_stamp;
                        let percent = delta_clocks as f64 / total_clocks as f64 * 100.0;
                        println!("{}: {} cycles ({:.2}%)", open_name, delta_clocks, percent);
                    } else { panic!("A profiler was unregistered without being registered."); } 
                }
            }
        }
        assert!(open_nodes.len() == 0, "A profiler was registered without being unregistered.");

        println!("Total: {} cycles ({} seconds)", total_clocks, total_secs);
    }
    pub fn register(&mut self, name: &'static str) {
        self.nodes.push(ProfilerNode::Open{ stamp: read_cpu_timer(), name: name });
    }
    pub fn unregister(&mut self, name: &'static str) {
        self.nodes.push(ProfilerNode::Close{ stamp: read_cpu_timer(), name: name });
    }
}

pub static mut PROFILER: Profiler = Profiler{ creation_stamp: 0, cpu_freq: 0, nodes: Vec::<ProfilerNode>::new() };

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

        unsafe { PROFILER.register(func_name); }

        let _defer_unregister = Defer::new(|| {
            unsafe { PROFILER.unregister(func_name); }
        });
    }
}

#[macro_export]
macro_rules! time_block {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        unsafe { PROFILER.register($msg); }

        let _defer_unregister = Defer::new(|| {
            unsafe { PROFILER.unregister($msg); }
        });
    }
}

#[macro_export]
macro_rules! time_stmt_rhsmsg  {
    ($x:stmt) => {
        let stmt = stringify!($x);
        let rhs = if let Some(equal_index) = stmt.find('=') {
            stmt[equal_index + 1..stmt.len()-1].trim_start()
        } else { &stmt[..stmt.len()-1] };
        unsafe { PROFILER.register(rhs); }
        $x;
        unsafe { PROFILER.unregister(rhs); }
    }
}

#[macro_export]
macro_rules! time_stmt_lhsmsg  {
    ($x:stmt) => {
        let stmt = stringify!($x);
        let lhs = if let Some(equal_index) = stmt.find('=') {
            if let Some(variable_name_index) = stmt[0..equal_index].trim_end().rfind(' ') {
                stmt[variable_name_index + 1..equal_index].trim() // simplify??
            } else { &stmt }
        } else { &stmt };
        unsafe { PROFILER.register(lhs); }
        $x;
        unsafe { PROFILER.unregister(lhs); }
    }
}

#[macro_export]
macro_rules! time_open {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        unsafe { PROFILER.register($msg); }
    }
}

#[macro_export]
macro_rules! time_close {
    // `()` indicates that the macro takes no argument.
    ( $msg:expr ) => {
        unsafe { PROFILER.unregister($msg); }
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

pub fn test() {
    profiler_setup();
    {
        time_block!("print hello block");
        println!("Hello, from non-macro println!");
    }
    profiler_teardown();
}