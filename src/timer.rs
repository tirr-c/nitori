use tokio_timer::{self, Timer};
use std::time::Duration;

lazy_static! {
    pub static ref TIMER: Timer = tokio_timer::wheel().build();
    pub static ref LONG_TIMER: Timer = tokio_timer::wheel().tick_duration(Duration::from_secs(1)).build();
}
