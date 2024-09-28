use std::time::{UNIX_EPOCH, Duration, SystemTime};

const BYE_N: usize = 7;
const BYE: [&str; BYE_N] = ["kthxbye", "aight, imma head out", "Bye, Felicia", "Goodbye, old friend", "Peace out!", "Smell you later!", "Toodle-oo"];

/// The important things in life
pub fn bye() -> &'static str {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let now = now.as_micros() as usize;
    let i = now % BYE_N;
    BYE[i]
}
