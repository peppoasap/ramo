//! Bounded, one-use previews and one in-flight provider request. No credentials.
use crate::{ai::AiInput, cancellation::Cancellation};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Default)]
struct State {
    previews: HashMap<String, AiInput>,
    active: HashMap<String, Cancellation>,
}
#[derive(Clone, Default)]
pub struct Requests(Arc<Mutex<State>>);
pub struct Running {
    pub input: AiInput,
    pub cancellation: Cancellation,
    owner: Requests,
}
impl Requests {
    pub fn insert(&self, input: AiInput) -> Result<(), String> {
        let mut state = self.0.lock().map_err(|_| "AI request state unavailable.")?;
        if state.previews.len() >= 8 {
            state.previews.clear();
        }
        state.previews.insert(input.id.clone(), input);
        Ok(())
    }
    pub fn begin(&self, id: &str) -> Result<Running, String> {
        let mut state = self.0.lock().map_err(|_| "AI request state unavailable.")?;
        if !state.active.is_empty() {
            return Err(
                "Another AI request is still finishing. Wait before preparing a new request."
                    .into(),
            );
        }
        let input = state
            .previews
            .remove(id)
            .ok_or("AI input expired or cancelled. Prepare it again.")?;
        let cancellation = Cancellation::default();
        state.active.insert(id.into(), cancellation.clone());
        Ok(Running {
            input,
            cancellation,
            owner: self.clone(),
        })
    }
    /// Removing an unclaimed preview also handles Stop arriving before Generate.
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let mut state = self.0.lock().map_err(|_| "AI request state unavailable.")?;
        state.previews.remove(id);
        if let Some(token) = state.active.get(id) {
            token.cancel();
        }
        Ok(())
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Ok(mut state) = self.owner.0.lock() {
            state.active.remove(&self.input.id);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input(id: &str) -> AiInput {
        AiInput {
            id: id.into(),
            repo_id: "synthetic".into(),
            fingerprint: "snapshot".into(),
            kind: "commit".into(),
            paths: vec![],
            excluded: vec![],
            text: "synthetic diff".into(),
        }
    }
    #[test]
    fn cancelling_an_unclaimed_preview_prevents_start() {
        let requests = Requests::default();
        requests.insert(input("one")).unwrap();
        requests.cancel("one").unwrap();
        assert!(requests.begin("one").is_err());
    }
    #[test]
    fn cancellation_reaches_only_its_job_and_cleanup_allows_a_fresh_preview() {
        let requests = Requests::default();
        requests.insert(input("one")).unwrap();
        requests.insert(input("two")).unwrap();
        let first = requests.begin("one").unwrap();
        assert!(requests.begin("two").is_err());
        requests.cancel("unknown").unwrap();
        assert!(!first.cancellation.is_cancelled());
        requests.cancel("one").unwrap();
        assert!(first.cancellation.check().is_err());
        assert!(requests.begin("one").is_err());
        drop(first);
        let second = requests.begin("two").unwrap();
        assert!(!second.cancellation.is_cancelled());
        drop(second);
        assert!(requests.begin("two").is_err());
    }
    #[test]
    fn job_failure_or_early_return_releases_its_slot_without_reusing_input() {
        let requests = Requests::default();
        requests.insert(input("one")).unwrap();
        drop(requests.begin("one").unwrap());
        assert!(requests.begin("one").is_err());
        requests.insert(input("new")).unwrap();
        assert!(requests.begin("new").is_ok());
    }
}
