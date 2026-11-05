//! Cooperative cancellation shared by a single AI request and its transport.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
pub const CANCELLED: &str = "AI request cancelled. Prepare a new input before retrying.";
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
    pub fn check(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err(CANCELLED.into())
        } else {
            Ok(())
        }
    }
}
