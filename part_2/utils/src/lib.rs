pub struct Defer<F> 
    where F: Fn() -> () { 
    op: F 
}

impl<F> Defer<F> 
    where F: Fn() -> () {
    pub fn new(op: F) -> Defer<F> {
        Defer { op: op }
    }
}

impl<F> Drop for Defer<F> 
    where F: Fn() -> () {
    fn drop(&mut self) { (self.op)(); }
}

#[cfg(test)]
mod tests {

    #[test]
    fn defer_counter_loop() {
        fn counter_inc() -> usize {
            static mut COUNTER: usize = 0;
            unsafe { COUNTER += 1; }
            unsafe { COUNTER }
        }

        let loop_count = 3;
        let _defer_push_func = super::Defer::new(|| {
            assert_eq!(counter_inc(), (loop_count * 2) + 1); 
        });
        for i in 0..loop_count {
            let _defer_push_loop = super::Defer::new(|| {
                assert_eq!(counter_inc(), (i * 2) + 2);
            });
            assert_eq!(counter_inc(), (i * 2) + 1);
            // _defer_push_loop exits scope each iteration of loop
        }
        // _defer_push_func exits scope when test exits scope
    }
}