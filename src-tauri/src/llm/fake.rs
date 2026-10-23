//! A [`Corrector`] for tests: no network, no API key.

use std::future::{Future, ready};

use super::{CorrectionError, Corrector};

/// Returns the same preset result on every call.
#[derive(Debug, Clone)]
pub struct FakeCorrector {
    result: Result<String, CorrectionError>,
}

impl FakeCorrector {
    /// A fake whose corrections always succeed with `text`.
    pub fn returning(text: impl Into<String>) -> Self {
        Self {
            result: Ok(text.into()),
        }
    }

    /// A fake whose corrections always fail with `error`.
    pub fn failing(error: CorrectionError) -> Self {
        Self { result: Err(error) }
    }
}

impl Corrector for FakeCorrector {
    fn correct(
        &self,
        _instructions: &str,
        _text: &str,
    ) -> impl Future<Output = Result<String, CorrectionError>> + Send {
        ready(self.result.clone())
    }
}
