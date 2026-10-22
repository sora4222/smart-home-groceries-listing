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

/// A store page the app shows in the store window. `Store::url` gives its
/// address and `StoreAbilities::allows` whether the app may show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorePage {
    /// The website, to log in.
    Home,
    /// The trolley page, where the fill program runs.
    Trolley,
    /// The checkout page, to check and pay.
    Checkout,
}

/// Every store, in the order the web app lists them.
pub const ALL: [Store; 2] = [Store::Woolworths, Store::Coles];

impl StoreAbilities {
    /// True when these abilities let the app show `page`.
    pub fn allows(self, page: StorePage) -> bool {
        match page {
            StorePage::Home => self.login,
            StorePage::Trolley => self.fill_trolley,
            StorePage::Checkout => self.checkout,
        }
    }
}

impl Store {
    /// `page`'s address at this store: the one place store pages are named.
    pub fn url(self, page: StorePage) -> &'static str {
        match (self, page) {
            (Store::Woolworths, StorePage::Home) => "https://www.woolworths.com.au/",
            (Store::Woolworths, StorePage::Trolley) => {
                "https://www.woolworths.com.au/shop/mytrolley"
            }
            (Store::Woolworths, StorePage::Checkout) => {
                "https://www.woolworths.com.au/shop/checkout"
            }
            (Store::Coles, StorePage::Home) => "https://www.coles.com.au/",
            (Store::Coles, StorePage::Trolley) => "https://www.coles.com.au/trolley",
            (Store::Coles, StorePage::Checkout) => "https://www.coles.com.au/checkout",
        }
    }

    /// True when `page` is this store's own trolley page (any query string),
    /// where the fill program may start.
    pub fn is_trolley_page(self, page: &Url) -> bool {
        let trolley =
            Url::parse(self.url(StorePage::Trolley)).expect("store trolley URLs are valid");
        self.owns(page) && page.path().trim_end_matches('/') == trolley.path()
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

    const PAGES: [StorePage; 3] = [StorePage::Home, StorePage::Trolley, StorePage::Checkout];

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
            for page in PAGES {
                assert!(store.owns(&url(store.url(page))), "{page:?}");
            }
        }
    }

    #[test]
    fn the_trolley_page_is_found_by_path_on_the_store_only() {
        let w = Store::Woolworths;
        assert!(w.is_trolley_page(&url(w.url(StorePage::Trolley))));
        assert!(w.is_trolley_page(&url(
            "https://www.woolworths.com.au/shop/mytrolley/?from=login"
        )));
        assert!(!w.is_trolley_page(&url(w.url(StorePage::Checkout))));
        assert!(!w.is_trolley_page(&url("http://www.woolworths.com.au/shop/mytrolley")));
        assert!(!Store::Coles.is_trolley_page(&url(w.url(StorePage::Trolley))));
    }

    #[test]
    fn each_page_pairs_its_address_with_its_ability() {
        let w = Store::Woolworths;
        assert_eq!(w.url(StorePage::Home), "https://www.woolworths.com.au/");
        assert_eq!(
            Store::Coles.url(StorePage::Checkout),
            "https://www.coles.com.au/checkout"
        );
        let only_login = StoreAbilities {
            store: w,
            login: true,
            fill_trolley: false,
            checkout: false,
        };
        assert!(only_login.allows(StorePage::Home));
        assert!(!only_login.allows(StorePage::Trolley));
        assert!(!only_login.allows(StorePage::Checkout));
    }

    #[test]
    fn only_woolworths_is_built() {
        assert!(Store::Woolworths.abilities().fill_trolley);
        let coles = Store::Coles.abilities();
        assert!(!coles.login && !coles.fill_trolley && !coles.checkout);
    }

    #[test]
    fn abilities_keep_the_shape_the_web_app_reads() {
        assert_eq!(
            serde_json::to_value(Store::Woolworths.abilities()).unwrap(),
            serde_json::json!({
                "store": "woolworths",
                "login": true,
                "fill_trolley": true,
                "checkout": true
            })
        );
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
