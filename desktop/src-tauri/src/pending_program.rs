//! The fill program waiting for a store's trolley page to finish loading.
//!
//! `fill_store_trolley` leaves the program here and sends the store window to
//! the trolley page. When a page finishes loading, the window asks for the
//! program; it is handed over only on that store's own trolley page, and only
//! once. A login page in between does not use it up, so after logging in and
//! landing on the trolley the fill still happens.

use std::collections::HashMap;
use std::sync::Mutex;

use url::Url;

use crate::stores::Store;

/// One waiting program per store. A newer send replaces an older one, as a
/// newer handoff replaces an older one on the server.
#[derive(Default)]
pub struct PendingPrograms(Mutex<HashMap<Store, String>>);

impl PendingPrograms {
    /// Leaves `program` for the next load of `store`'s trolley page.
    pub fn set(&self, store: Store, program: String) {
        let replaced = self.lock().insert(store, program).is_some();
        if replaced {
            log::info!("[store] newer fill program replaced a waiting one for {store:?}");
        }
    }

    /// The program for `store`, if `page` is that store's trolley page.
    /// Removes it, so it runs once.
    pub fn take_for(&self, store: Store, page: &Url) -> Option<String> {
        if !is_trolley_page(store, page) {
            return None;
        }
        self.lock().remove(&store)
    }

    /// True while a program is waiting for `store`.
    #[cfg(test)]
    pub fn is_waiting(&self, store: Store) -> bool {
        self.lock().contains_key(&store)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<Store, String>> {
        // A panic while holding the lock leaves plain data behind; keep going.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// True when `page` is `store`'s own trolley page (any query string).
fn is_trolley_page(store: Store, page: &Url) -> bool {
    let trolley = Url::parse(store.trolley_url()).expect("store trolley URLs are valid");
    store.owns(page) && page.path().trim_end_matches('/') == trolley.path()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    const TROLLEY: &str = "https://www.woolworths.com.au/shop/mytrolley";

    #[test]
    fn hands_the_program_over_once_on_the_trolley_page() {
        let pending = PendingPrograms::default();
        pending.set(Store::Woolworths, "fill()".into());
        assert_eq!(
            pending.take_for(Store::Woolworths, &url(TROLLEY)),
            Some("fill()".into())
        );
        assert_eq!(pending.take_for(Store::Woolworths, &url(TROLLEY)), None);
    }

    #[test]
    fn a_login_page_does_not_use_it_up() {
        let pending = PendingPrograms::default();
        pending.set(Store::Woolworths, "fill()".into());
        let login = url("https://www.woolworths.com.au/shop/securelogin");
        assert_eq!(pending.take_for(Store::Woolworths, &login), None);
        assert!(pending.is_waiting(Store::Woolworths));
        let back = url("https://www.woolworths.com.au/shop/mytrolley/?from=login");
        assert_eq!(
            pending.take_for(Store::Woolworths, &back),
            Some("fill()".into())
        );
    }

    #[test]
    fn never_runs_on_another_website() {
        let pending = PendingPrograms::default();
        pending.set(Store::Woolworths, "fill()".into());
        let elsewhere = url("https://evil.example/shop/mytrolley");
        assert_eq!(pending.take_for(Store::Woolworths, &elsewhere), None);
        assert_eq!(pending.take_for(Store::Coles, &url(TROLLEY)), None);
    }

    #[test]
    fn a_newer_program_replaces_the_older() {
        let pending = PendingPrograms::default();
        pending.set(Store::Woolworths, "old()".into());
        pending.set(Store::Woolworths, "new()".into());
        assert_eq!(
            pending.take_for(Store::Woolworths, &url(TROLLEY)),
            Some("new()".into())
        );
    }
}
