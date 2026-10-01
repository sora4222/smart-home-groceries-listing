//! Order review's log events, one function per thing that happened.
//!
//! Item ids, stores, product ids and prices are logged. Nothing here is a
//! secret.

use super::line::OrderLine;
use super::summary::OrderReview;

/// A household member opened or refreshed the order review.
pub fn reviewed(review: &OrderReview) {
    tracing::info!(
        stores = review.stores.len(),
        lines = review.stores.iter().map(|s| s.lines.len()).sum::<usize>(),
        unchosen = review.unchosen.len(),
        problems = review.problem_count(),
        total = %review.total,
        complete = review.complete,
        "order reviewed"
    );
}

/// One chosen product could not be re-priced as it is.
pub fn line_problem(line: &OrderLine) {
    tracing::warn!(
        item_id = %line.item.id,
        store = %line.choice.store,
        product_id = %line.choice.product_id,
        status = ?line.status,
        problem = line.problem.as_deref().unwrap_or_default(),
        "order line needs attention"
    );
}

/// One chosen product costs something different from when it was chosen.
pub fn price_changed(line: &OrderLine) {
    tracing::info!(
        item_id = %line.item.id,
        store = %line.choice.store,
        product_id = %line.choice.product_id,
        then = ?line.choice.price,
        now = ?line.price,
        "chosen product's price changed"
    );
}
