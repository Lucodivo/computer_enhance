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

pub fn printable_freq(freq: u64) -> String {
    match freq {
        0..=999 => format!("{} Hz", freq),
        1_000..=999_999 => format!("{:.3} KHz", (freq as f64) / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.2} MHz", (freq as f64) / 1_000_000.0),
        1_000_000_000..=999_999_999_999 => format!("{:.1} GHz", (freq as f64) / 1_000_000_000.0),
        _ => format!("{:.1} THz", (freq as f64) / 1_000_000_000_000.0)
    }
}

pub fn printable_large_num(number: u64) -> String {
    number.to_string()
        .as_bytes()
        .rchunks(3).rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
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