//! The rules for talking to the store tab: what the bookmarklet is sent, and
//! how its report is checked and turned into a final status.
//!
//! Pure functions — no database, no HTTP — so every rule is unit-tested here.
//!
//! The store's trolley call sets a product's quantity rather than adding to
//! it, and two list items can share one chosen product, so the store tab is
//! sent one line per *product* with the items' quantities summed.

use std::collections::{HashMap, HashSet};

use crate::error::ApiError;
use crate::models::db::{TrolleyHandoffLine, TrolleyHandoffStatus, TrolleyLineOutcome};

/// One product for the store tab to put in the trolley.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTabLine {
    pub product_id: String,
    pub product_name: String,
    /// How many to add on top of what is already in the trolley.
    pub quantity: i32,
}

/// What the store tab says happened to one product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedLine {
    pub product_id: String,
    pub outcome: TrolleyLineOutcome,
    pub problem: Option<String>,
}

/// The handoff's lines as the store tab needs them: one per product, in list
/// order, quantities of a shared product summed.
pub fn lines_for_store_tab(lines: &[TrolleyHandoffLine]) -> Vec<StoreTabLine> {
    let mut merged: Vec<StoreTabLine> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();
    for line in lines {
        match index.get(line.product_id.as_str()) {
            Some(&at) => merged[at].quantity += line.quantity,
            None => {
                index.insert(&line.product_id, merged.len());
                merged.push(StoreTabLine {
                    product_id: line.product_id.clone(),
                    product_name: line.product_name.clone(),
                    quantity: line.quantity,
                });
            }
        }
    }
    merged
}

/// Checks a report covers exactly the handoff's products, each once.
///
/// A report naming a product the handoff never asked for, naming one twice,
/// or leaving one out is refused (422): the household would otherwise be
/// told the trolley was filled when it was not.
pub fn check_report(lines: &[TrolleyHandoffLine], report: &[ReportedLine]) -> Result<(), ApiError> {
    let expected: HashSet<&str> = lines.iter().map(|l| l.product_id.as_str()).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for line in report {
        let id = line.product_id.as_str();
        if !expected.contains(id) {
            return Err(ApiError::UnprocessableEntity(format!(
                "Product {id} is not part of this handoff."
            )));
        }
        if !seen.insert(id) {
            return Err(ApiError::UnprocessableEntity(format!(
                "Product {id} is reported more than once."
            )));
        }
    }
    if seen.len() != expected.len() {
        return Err(ApiError::UnprocessableEntity(
            "The report must say what happened to every product.".to_string(),
        ));
    }
    Ok(())
}

/// The status a handoff ends with once its report is in.
pub fn status_after_report(report: &[ReportedLine]) -> TrolleyHandoffStatus {
    if report
        .iter()
        .all(|l| l.outcome == TrolleyLineOutcome::Added)
    {
        TrolleyHandoffStatus::Filled
    } else {
        TrolleyHandoffStatus::FilledWithProblems
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn line(product_id: &str, quantity: i32) -> TrolleyHandoffLine {
        TrolleyHandoffLine {
            handoff_id: Uuid::nil(),
            grocery_item_id: Uuid::new_v4(),
            product_id: product_id.into(),
            product_name: format!("product {product_id}"),
            quantity,
            outcome: None,
            problem: None,
        }
    }

    fn reported(product_id: &str, outcome: TrolleyLineOutcome) -> ReportedLine {
        ReportedLine {
            product_id: product_id.into(),
            outcome,
            problem: None,
        }
    }

    #[test]
    fn a_product_chosen_for_two_items_is_sent_once_with_both_quantities() {
        let sent = lines_for_store_tab(&[line("1", 2), line("2", 1), line("1", 3)]);
        assert_eq!(sent.len(), 2);
        assert_eq!((sent[0].product_id.as_str(), sent[0].quantity), ("1", 5));
        assert_eq!((sent[1].product_id.as_str(), sent[1].quantity), ("2", 1));
    }

    #[test]
    fn a_complete_report_is_accepted() {
        let lines = [line("1", 1), line("2", 1)];
        let report = [
            reported("2", TrolleyLineOutcome::Added),
            reported("1", TrolleyLineOutcome::Failed),
        ];
        assert!(check_report(&lines, &report).is_ok());
    }

    #[test]
    fn unknown_repeated_or_missing_products_are_refused() {
        let lines = [line("1", 1), line("2", 1)];
        let added = TrolleyLineOutcome::Added;
        assert!(check_report(&lines, &[reported("1", added), reported("9", added)]).is_err());
        assert!(check_report(&lines, &[reported("1", added), reported("1", added)]).is_err());
        assert!(check_report(&lines, &[reported("1", added)]).is_err());
    }

    #[test]
    fn any_failed_line_means_filled_with_problems() {
        let all_added = [reported("1", TrolleyLineOutcome::Added)];
        let one_failed = [
            reported("1", TrolleyLineOutcome::Added),
            reported("2", TrolleyLineOutcome::Failed),
        ];
        assert_eq!(
            status_after_report(&all_added),
            TrolleyHandoffStatus::Filled
        );
        assert_eq!(
            status_after_report(&one_failed),
            TrolleyHandoffStatus::FilledWithProblems
        );
    }
}
