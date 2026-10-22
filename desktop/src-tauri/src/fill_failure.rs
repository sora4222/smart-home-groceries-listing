//! Every way a trolley fill can end without a report, and the one sentence a
//! person reads for each. The fill command turns it into text only at the
//! IPC boundary (`commands/store.rs`).

use std::fmt;

use crate::stores::Store;

/// Every way a fill can end without a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillFailure {
    /// Another fill is already going at this store.
    Busy(Store),
    /// The store window closed first.
    WindowClosed,
    /// The store page loaded again and lost the program's progress.
    PageChanged,
    /// The store window refused to run the program.
    CouldNotStart,
    /// The fill took longer than the command waits.
    TimedOut,
    /// The operation ended without saying how.
    Stopped,
    /// The program's own sentence from the store page.
    Program(String),
}

/// The one place the sentences a person reads about a failed fill live.
impl fmt::Display for FillFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FillFailure::Busy(store) => write!(
                f,
                "A trolley fill is already running at {}. Wait for it to finish.",
                store.display_name()
            ),
            FillFailure::WindowClosed => {
                f.write_str("The store window was closed before the trolley was filled.")
            }
            FillFailure::PageChanged => {
                f.write_str("The store page changed before the trolley was filled. Try again.")
            }
            FillFailure::CouldNotStart => {
                f.write_str("The fill could not start in the store window.")
            }
            FillFailure::TimedOut => f.write_str("The trolley fill took too long and was stopped."),
            FillFailure::Stopped => f.write_str("The trolley fill stopped unexpectedly."),
            FillFailure::Program(message) => f.write_str(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_failure_says_what_happened_in_one_sentence() {
        let sentences = [
            FillFailure::Busy(Store::Coles).to_string(),
            FillFailure::WindowClosed.to_string(),
            FillFailure::PageChanged.to_string(),
            FillFailure::CouldNotStart.to_string(),
            FillFailure::TimedOut.to_string(),
            FillFailure::Stopped.to_string(),
        ];
        assert!(sentences[0].contains("Coles"));
        for sentence in &sentences {
            assert!(sentence.ends_with('.'), "{sentence}");
        }
        let unique: std::collections::HashSet<_> = sentences.iter().collect();
        assert_eq!(unique.len(), sentences.len());
        assert_eq!(
            FillFailure::Program("Fill trolley failed: 401".into()).to_string(),
            "Fill trolley failed: 401"
        );
    }
}
