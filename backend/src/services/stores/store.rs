//! The retailers the household can buy from.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A supermarket with online ordering and home delivery.
///
/// Aldi is deliberately absent: it has no online ordering in Australia.
///
/// Stored as lowercase `TEXT` (`item_selections.store`), and read from request
/// bodies, so an unknown store name is refused before it reaches a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Woolworths,
    Coles,
}

impl Store {
    /// Every store, in the order results are shown.
    pub const ALL: [Store; 2] = [Store::Woolworths, Store::Coles];

    /// The lowercase identifier used in JSON and in logs.
    pub fn as_str(self) -> &'static str {
        match self {
            Store::Woolworths => "woolworths",
            Store::Coles => "coles",
        }
    }

    /// The name a person reads.
    pub fn display_name(self) -> &'static str {
        match self {
            Store::Woolworths => "Woolworths",
            Store::Coles => "Coles",
        }
    }
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_lowercase() {
        assert_eq!(
            serde_json::to_value(Store::Woolworths).unwrap(),
            "woolworths"
        );
        assert_eq!(Store::Coles.to_string(), "coles");
    }

    #[test]
    fn reads_lowercase_and_refuses_other_stores() {
        assert_eq!(
            serde_json::from_value::<Store>(serde_json::json!("coles")).unwrap(),
            Store::Coles
        );
        assert!(serde_json::from_value::<Store>(serde_json::json!("aldi")).is_err());
    }
}
