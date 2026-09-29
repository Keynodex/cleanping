//! A history that remembers nothing, for `--no-history` and for `edit`.

use cleanping_core::application::ports::RunRepository;
use cleanping_core::domain::errors::Result;
use cleanping_core::domain::models::Run;

pub struct NoHistory;

impl RunRepository for NoHistory {
    fn add(&self, run: &Run) -> Result<Run> {
        Ok(run.clone())
    }
    fn recent(&self, _limit: usize) -> Result<Vec<Run>> {
        Ok(Vec::new())
    }
    fn delete_all(&self) -> Result<usize> {
        Ok(0)
    }
    fn delete_before(&self, _cutoff: &str) -> Result<usize> {
        Ok(0)
    }
}
