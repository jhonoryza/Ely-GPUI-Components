mod clock;
mod stopwatch;
#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use clock::{Clock, WorldClock};
pub use stopwatch::Stopwatch;
