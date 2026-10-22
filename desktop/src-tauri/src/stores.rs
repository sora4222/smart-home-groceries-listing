//! The stores the desktop app can open, their addresses, and what the app can
//! do at each one yet.
//!
//! Woolworths addresses match the bookmark flow (`FEATURE_TROLLEY_HANDOFF.md`).
//! Coles is listed so the web app can ask about it, but every ability is off
//! until its trolley calls are captured (`FEATURE_DESKTOP_APP.md`, goal 9).

use serde::{Deserialize, Serialize};
use url::Url;

/// A store, named as the backend and web app name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Woolworths,
    Coles,
}

/// What the desktop app can do at a store today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StoreAbilities {
    pub store: Store,
    /// Open the store window to log in.
    pub login: bool,
    /// Run the fill program in the store window.
    pub fill_trolley: bool,
    /// Show the store's checkout page.
    pub checkout: bool,
}

/// Every store, in the order the web app lists them.
pub const ALL: [Store; 2] = [Store::Woolworths, Store::Coles];

impl Store {
    /// The store's website, for logging in.
    pub fn home_url(self) -> &'static str {
        match self {
            Store::Woolworths => "https://www.woolworths.com.au/",
            Store::Coles => "https://www.coles.com.au/",
        }
    }

    /// The trolley page, where the fill program runs.
    pub fn trolley_url(self) -> &'static str {
        match self {
            Store::Woolworths => "https://www.woolworths.com.au/shop/mytrolley",
            Store::Coles => "https://www.coles.com.au/trolley",
        }
    }

    /// The checkout page, to check and pay.
    pub fn checkout_url(self) -> &'static str {
        match self {
            Store::Woolworths => "https://www.woolworths.com.au/shop/checkout",
            Store::Coles => "https://www.coles.com.au/checkout",
        }
    }

    /// The label of this store's window. One window per store, so its login
    /// cookies stay in one place.
    pub fn window_label(self) -> &'static str {
        match self {
            Store::Woolworths => "store-woolworths",
            Store::Coles => "store-coles",
        }
    }

    /// The window title the person sees.
    pub fn display_name(self) -> &'static str {
        match self {
            Store::Woolworths => "Woolworths",
            Store::Coles => "Coles",
        }
    }

    /// What the app can do here today.
    pub fn abilities(self) -> StoreAbilities {
        let built = self == Store::Woolworths;
        StoreAbilities {
            store: self,
            login: built,
            fill_trolley: built,
            checkout: built,
        }
    }

    /// True when `url` is a secure page on this store's own website — the
    /// only place the fill program may run.
    pub fn owns(self, url: &Url) -> bool {
        let domain = match self {
            Store::Woolworths => "woolworths.com.au",
            Store::Coles => "coles.com.au",
        };
        url.scheme() == "https"
            && url
                .host_str()
                .is_some_and(|host| host == domain || host.ends_with(&format!(".{domain}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn woolworths_owns_its_own_secure_pages_only() {
        let w = Store::Woolworths;
        assert!(w.owns(&url("https://www.woolworths.com.au/shop/mytrolley")));
        assert!(w.owns(&url("https://woolworths.com.au/")));
        assert!(!w.owns(&url("http://www.woolworths.com.au/")));
        assert!(!w.owns(&url("https://woolworths.com.au.evil.example/")));
        assert!(!w.owns(&url("https://notwoolworths.com.au/")));
        assert!(!w.owns(&url("https://www.coles.com.au/")));
    }

    #[test]
    fn every_store_page_belongs_to_its_store() {
        for store in ALL {
            for page in [store.home_url(), store.trolley_url(), store.checkout_url()] {
                assert!(store.owns(&url(page)), "{page}");
            }
        }
    }

    #[test]
    fn only_woolworths_is_built() {
        assert!(Store::Woolworths.abilities().fill_trolley);
        let coles = Store::Coles.abilities();
        assert!(!coles.login && !coles.fill_trolley && !coles.checkout);
    }

    #[test]
    fn names_match_the_web_app() {
        assert_eq!(
            serde_json::to_string(&Store::Woolworths).unwrap(),
            "\"woolworths\""
        );
        assert_eq!(
            serde_json::from_str::<Store>("\"coles\"").unwrap(),
            Store::Coles
        );
    }
}
