//! LLM triage: is an intake item something a supermarket sells?
//!
//! Every intake request is recorded `unchecked`, then classified in the
//! background ([`TriageQueue`]). The answer only picks the queue a person sees
//! it in — Pending Requests (`approved`), or the Triage view's Held for review
//! (`held`) or Rejected (`rejected`) tabs. **Triage never accepts an item**:
//! a household member always does. See `docs/features/FEATURE_TRIAGE.md`.
//!
//! | File | Job |
//! |---|---|
//! | `model.rs` | the [`TriageModel`] trait every classifier implements |
//! | `openai.rs` | Ollama, llama.cpp, vLLM and OpenAI — one chat-completions client |
//! | `fake.rs` | keyword classifier for development and tests |
//! | `prompt.rs` | the instructions, and reading the reply (pure) |
//! | `verdict.rs` | answer → status, reason, confidence (pure) |
//! | `assessor.rs` | one classification, with a time limit |
//! | `registry.rs` | picks the classifier from `INTAKE_LLM_PROVIDER` |
//! | `queue.rs` | background checks after an item is recorded |
//! | `review.rs` | the Triage view's tabs and its "move to pending" |

mod assessor;
mod fake;
mod log;
mod model;
mod openai;
mod prompt;
mod queue;
pub mod registry;
mod review;
mod verdict;

pub use assessor::Triage;
pub use fake::FakeTriage;
pub use model::{Classification, TriageError, TriageModel};
pub use openai::OpenAiCompatible;
pub use queue::TriageQueue;
pub use review::{TriageReview, TriageTab};
pub use verdict::{decide, TriageOutcome};
