use clocks::{measure_cpu_freq, read_cpu_timer, clocks_to_millisecs};
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
        let total_millisecs = clocks_to_millisecs(total_clocks, self.cpu_freq);

        println!("\n====== Profiler Results *START* =======\n");
        
        let measurements = self.nodes.len() / 2;
        let mut print_strings = Vec::<String>::with_capacity(measurements);

        let mut prefix = String::with_capacity(100);
        let mut open_nodes: Vec<&ProfilerNode> = Vec::new();
        let prefix_str_token = "  ";
        for node in &self.nodes {
            match node {
                ProfilerNode::Open{ .. } => {
                    if open_nodes.len() != 0 { prefix.push_str(prefix_str_token); }
                    open_nodes.push(&node);
                },
                ProfilerNode::Close{ stamp: close_stamp, name: close_name } => {
                    if let ProfilerNode::Open{ stamp: open_stamp, name: open_name } = open_nodes.pop().expect("A profiler was unregistered without being registered.") {
                        assert!(open_name == close_name, "Unmatched names for register and unregister found.");
                        let delta_clocks = close_stamp - open_stamp;
                        let percent = delta_clocks as f64 / total_clocks as f64 * 100.0;
                        let title_str = format!("{}:", open_name);
                        let clocks_str = format!("{} cycles", delta_clocks);
                        let percent_str = format!("({:.2}%)", percent);
                        print_strings.push(format!("{}{:<40}{:<30}{}", prefix, title_str, clocks_str, percent_str).to_string());
                        if prefix.len() > 0 { for _ in 0..prefix_str_token.len() { prefix.pop(); } }
                    } else { panic!("A profiler was unregistered without being registered."); }
                    if open_nodes.len() == 0 { // found a root node
                        for s in print_strings.iter().rev() {
                            println!("{}", s);
                        }
                        print_strings.clear();
                    }
                }
            }
        }
        assert!(open_nodes.len() == 0, "A profiler was registered without being unregistered.");

        let title_str = "Total runtime:";
        let time_str = format!("{} cycles ({:.3}ms)", total_clocks, total_millisecs);
        println!("{:<40}{:<30}", title_str, time_str);

        println!("\n====== Profiler Results *END* =======\n");
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
macro_rules! time_assignment_rhs  {
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
macro_rules! time_assignment  {
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