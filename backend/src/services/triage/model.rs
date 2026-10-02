//! The contract every triage classifier satisfies.
//!
//! The rest of the backend sees only [`TriageModel`] — never a provider's URL,
//! key or JSON — so swapping OpenAI for Ollama, or for something else later,
//! is a setting, not a code change.

use crate::services::stores::client::BoxFuture;

/// What a classifier said about one intake item.
#[derive(Debug, Clone, PartialEq)]
pub struct Classification {
    /// Would a supermarket plausibly sell this?
    pub is_supermarket_item: bool,
    /// How sure the classifier is, 0 to 1.
    pub confidence: f32,
    /// One sentence saying why, shown to the household in the Triage view.
    pub reason: String,
}

/// Why a classifier could not answer. The message is shown to the household,
/// so it names what went wrong without any key, header or response body.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TriageError {
    /// The classifier could not be reached, refused, or took too long.
    #[error("the checker could not be reached ({0})")]
    Unavailable(String),
    /// It answered, but not in the shape asked for.
    #[error("the checker gave an answer that could not be read")]
    BadReply,
}

/// One classifier: is this text something a supermarket sells?
pub trait TriageModel: Send + Sync {
    /// Classifies the item text exactly as the intake channel heard it.
    fn classify<'a>(
        &'a self,
        item_text: &'a str,
    ) -> BoxFuture<'a, Result<Classification, TriageError>>;
}
