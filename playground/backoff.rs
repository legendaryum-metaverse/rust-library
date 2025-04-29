use backoff::ExponentialBackoff;
use std::io::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn pseudo_random() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    (nanos % 1000) as f32 / 1000.0
}
fn fallible_operation() -> Result<(), Error> {
    if pseudo_random() < 0.8 {
        // Simulating a failure
        Err(Error::other("Operation failed"))
    } else {
        Ok(())
    }
}
// exponential_backoff until success
fn main() -> Result<(), backoff::Error<Error>> {
    let retry_count = AtomicUsize::new(0);

    let operation = || -> Result<(), backoff::Error<Error>> {
        retry_count.fetch_add(1, Ordering::SeqCst);
        fallible_operation().map_err(backoff::Error::transient)
    };

    let result = backoff::retry(ExponentialBackoff::default(), operation);

    println!(
        "Number of retries until ok: {}",
        retry_count.load(Ordering::SeqCst)
    );
    result
}
