use clocks;

struct Defer<F> 
    where F: Fn() -> () 
{ 
    op: F 
}

impl<F> Drop for Defer<F> 
    where F: Fn() -> () 
{
    fn drop(&mut self) { (self.op)(); }
}

/*
Calling this macro twice in the same function will *NOT* compile.
*/
macro_rules! defer_print_func_name {
    // `()` indicates that the macro takes no argument.
    () => {
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let func_name_start = &name[..name.len() - 3].rfind(':').expect("Failed to find function name.") + 1;
        let func_name_end = name.len() - 3; // remove the trailing "::f"
        let _defer_print = Defer { op: || { println!("{} has exited scope.", &name[func_name_start..func_name_end]); } }; 
    }
}

fn profiler_setup() {
    defer_print_func_name!();
}

fn profiler_teardown() {
    defer_print_func_name!();
}

pub fn test() {
    profiler_setup();
    defer_print_func_name!();
    {
        println!("Hello, from non-macro println!");
    }
    profiler_teardown();
}