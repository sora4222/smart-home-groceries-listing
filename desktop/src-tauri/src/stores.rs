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

/// A store page the app shows in the store window, with the ability it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorePage {
    /// The website, to log in.
    Home,
    /// The trolley page, where the fill program runs.
    Trolley,
    /// The checkout page, to check and pay.
    Checkout,
}

impl StorePage {
    /// This page's address at `store`.
    pub fn url(self, store: Store) -> &'static str {
        match self {
            StorePage::Home => store.home_url(),
            StorePage::Trolley => store.trolley_url(),
            StorePage::Checkout => store.checkout_url(),
        }
    }

    /// True when `abilities` let the app show this page.
    pub fn allowed(self, abilities: StoreAbilities) -> bool {
        match self {
            StorePage::Home => abilities.login,
            StorePage::Trolley => abilities.fill_trolley,
            StorePage::Checkout => abilities.checkout,
        }
    }
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

    /// True when `page` is this store's own trolley page (any query string),
    /// where the fill program may start.
    pub fn is_trolley_page(self, page: &Url) -> bool {
        let trolley =
            Url::parse(StorePage::Trolley.url(self)).expect("store trolley URLs are valid");
        self.owns(page) && page.path().trim_end_matches('/') == trolley.path()
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
    fn the_trolley_page_is_found_by_path_on_the_store_only() {
        let w = Store::Woolworths;
        assert!(w.is_trolley_page(&url(w.trolley_url())));
        assert!(w.is_trolley_page(&url(
            "https://www.woolworths.com.au/shop/mytrolley/?from=login"
        )));
        assert!(!w.is_trolley_page(&url(w.checkout_url())));
        assert!(!w.is_trolley_page(&url("http://www.woolworths.com.au/shop/mytrolley")));
        assert!(!Store::Coles.is_trolley_page(&url(w.trolley_url())));
    }

    #[test]
    fn each_page_pairs_its_address_with_its_ability() {
        let w = Store::Woolworths;
        assert_eq!(StorePage::Home.url(w), w.home_url());
        assert_eq!(StorePage::Trolley.url(w), w.trolley_url());
        assert_eq!(StorePage::Checkout.url(w), w.checkout_url());
        let only_login = StoreAbilities {
            store: w,
            login: true,
            fill_trolley: false,
            checkout: false,
        };
        assert!(StorePage::Home.allowed(only_login));
        assert!(!StorePage::Trolley.allowed(only_login));
        assert!(!StorePage::Checkout.allowed(only_login));
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
