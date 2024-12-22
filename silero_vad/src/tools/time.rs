pub fn timed<T>(name: &str, func: impl FnOnce() -> T) -> T {
    let start = std::time::Instant::now();
    let res = func();
    let elapsed = start.elapsed();
    println!("Elapsed time for function {}: {:?}", name, elapsed);
    res
}
