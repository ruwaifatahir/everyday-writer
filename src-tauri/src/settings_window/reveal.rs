//! When the settings window, built hidden, is shown: once, on the page's
//! ready signal or the safety timeout, whichever comes first.

/// One build of the settings window, so a timeout can tell its own window
/// from one opened after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening(u64);

/// Tracks the settings window from build to reveal to close.
#[derive(Debug)]
pub struct Reveal {
    opening: u64,
    hidden: bool,
}

impl Reveal {
    /// No window yet.
    pub const fn new() -> Self {
        Self {
            opening: 0,
            hidden: false,
        }
    }

    /// A window is being built hidden. Pass the result to [`Self::time_out`]
    /// and [`Self::close`].
    pub fn open(&mut self) -> Opening {
        self.opening += 1;
        self.hidden = true;
        Opening(self.opening)
    }

    /// The page is drawn. True if the window should be shown now.
    pub fn ready(&mut self) -> bool {
        std::mem::take(&mut self.hidden)
    }

    /// The safety timeout for `opening` ran out. True if the window should be
    /// shown now.
    pub fn time_out(&mut self, opening: Opening) -> bool {
        self.is_current(opening) && self.ready()
    }

    /// Whether a window was built and not shown yet.
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// The window built at `opening` closed. A late close of an earlier
    /// window leaves the current one alone.
    pub fn close(&mut self, opening: Opening) {
        if self.is_current(opening) {
            self.hidden = false;
        }
    }

    fn is_current(&self, opening: Opening) -> bool {
        opening == Opening(self.opening)
    }
}

#[cfg(test)]
mod tests {
    use super::Reveal;

    #[test]
    fn the_ready_signal_reveals_the_window() {
        let mut reveal = Reveal::new();
        reveal.open();

        assert!(reveal.ready());
    }

    #[test]
    fn the_timeout_reveals_the_window_if_the_page_never_got_ready() {
        let mut reveal = Reveal::new();
        let opening = reveal.open();

        assert!(reveal.time_out(opening));
    }

    #[test]
    fn the_timeout_does_nothing_after_the_ready_signal() {
        let mut reveal = Reveal::new();
        let opening = reveal.open();
        reveal.ready();

        assert!(!reveal.time_out(opening));
    }

    #[test]
    fn a_late_ready_signal_does_nothing_after_the_timeout() {
        let mut reveal = Reveal::new();
        let opening = reveal.open();
        reveal.time_out(opening);

        assert!(!reveal.ready());
    }

    #[test]
    fn the_timeout_does_nothing_after_the_window_closed() {
        let mut reveal = Reveal::new();
        let opening = reveal.open();
        reveal.close(opening);

        assert!(!reveal.time_out(opening));
    }

    #[test]
    fn a_closed_windows_timeout_leaves_a_reopened_window_hidden() {
        let mut reveal = Reveal::new();
        let first = reveal.open();
        reveal.close(first);
        reveal.open();

        assert!(!reveal.time_out(first));
    }

    #[test]
    fn a_closed_windows_late_close_leaves_a_reopened_window_hidden() {
        let mut reveal = Reveal::new();
        let first = reveal.open();
        reveal.open();
        reveal.close(first);

        assert!(reveal.is_hidden());
    }
}
