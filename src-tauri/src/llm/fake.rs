//! A [`Rewriter`] for tests: no network, no API key.

use std::future::{Future, ready};

use super::{RewriteError, Rewriter};

/// Returns the same preset result on every call.
#[derive(Debug, Clone)]
pub struct FakeRewriter {
    result: Result<String, RewriteError>,
}

impl FakeRewriter {
    /// A fake whose rewrites always succeed with `text`.
    pub fn returning(text: impl Into<String>) -> Self {
        Self {
            result: Ok(text.into()),
        }
    }

    /// A fake whose rewrites always fail with `error`.
    pub fn failing(error: RewriteError) -> Self {
        Self { result: Err(error) }
    }
}

impl Rewriter for FakeRewriter {
    fn rewrite(
        &self,
        _instructions: &str,
        _text: &str,
    ) -> impl Future<Output = Result<String, RewriteError>> + Send {
        ready(self.result.clone())
    }
}
