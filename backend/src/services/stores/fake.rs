//! A pretend store with a small, fixed catalogue.
//!
//! Selected with `STORE_CLIENTS=fake`. Development, the end-to-end tests and
//! any machine that must not reach a real store use it, so a run never
//! depends on Woolworths' or Coles' websites — or gets an address flagged by
//! their bot protection.
//!
//! A product is returned when the query contains its keyword ("toilet paper
//! 3 ply" returns every toilet paper; filter chips narrow it afterwards).
//! A query containing [`OUTAGE_KEYWORD`] makes the fake Coles fail as
//! unreachable, so the "one store is down" path can be exercised end to end.
//! A product is looked up by its catalogue id (`c-milk-3l`); the fake ids
//! are not digits, so they can never be mistaken for a real store's.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::client::{BoxFuture, StoreClient};
use super::deal::Deal;
use super::error::StoreError;
use super::measure;
use super::product::Product;
use super::store::Store;
use super::unit_price::normalise;

/// Searching for anything containing this makes the fake Coles unreachable.
pub const OUTAGE_KEYWORD: &str = "outage";

/// One catalogue row: the keyword that finds it, then the product's details.
struct Row {
    store: Store,
    keyword: &'static str,
    id: &'static str,
    brand: &'static str,
    name: &'static str,
    size: &'static str,
    price: Decimal,
    was: Option<Decimal>,
    /// Unit price and the measure it is quoted against.
    unit: (Decimal, &'static str),
    /// A multibuy: group size and per-unit price inside a group.
    deal: Option<(u32, Decimal)>,
    category: &'static str,
}

/// The catalogue. Small on purpose: enough to show cheaper-per-unit,
/// specials, multibuys, a converted unit price and an unavailable store.
const CATALOGUE: &[Row] = &[
    Row {
        store: Store::Woolworths,
        keyword: "milk",
        id: "w-milk-2l",
        brand: "Woolworths",
        name: "Full Cream Milk",
        size: "2L",
        price: dec!(3.10),
        was: None,
        unit: (dec!(1.55), "1L"),
        deal: None,
        category: "Milk",
    },
    Row {
        store: Store::Woolworths,
        keyword: "milk",
        id: "w-milk-3l",
        brand: "Dairy Farmers",
        name: "Full Cream Milk",
        size: "3L",
        price: dec!(5.40),
        was: Some(dec!(6.20)),
        unit: (dec!(1.80), "1L"),
        deal: None,
        category: "Milk",
    },
    Row {
        store: Store::Coles,
        keyword: "milk",
        id: "c-milk-3l",
        brand: "Coles",
        name: "Full Cream Milk",
        size: "3L",
        price: dec!(4.95),
        was: None,
        unit: (dec!(1.65), "1L"),
        deal: None,
        category: "Milk",
    },
    Row {
        store: Store::Coles,
        keyword: "milk",
        id: "c-oat-1l",
        brand: "Minor Figures",
        name: "Barista Oat Milk",
        size: "1L",
        price: dec!(3.90),
        was: None,
        unit: (dec!(3.90), "1L"),
        deal: Some((2, dec!(3.25))),
        category: "Long-Life Milk",
    },
    Row {
        store: Store::Woolworths,
        keyword: "toilet paper",
        id: "w-tp-3ply",
        brand: "Kleenex",
        name: "Complete Clean 3 Ply Toilet Paper",
        size: "12 pack",
        price: dec!(12.00),
        was: Some(dec!(13.50)),
        unit: (dec!(0.28), "100 sheets"),
        deal: None,
        category: "Toilet Paper",
    },
    Row {
        store: Store::Woolworths,
        keyword: "toilet paper",
        id: "w-tp-2ply",
        brand: "Woolworths",
        name: "Essentials 2 Ply Toilet Paper",
        size: "24 pack",
        price: dec!(9.00),
        was: None,
        unit: (dec!(0.19), "100 sheets"),
        deal: None,
        category: "Toilet Paper",
    },
    Row {
        store: Store::Coles,
        keyword: "toilet paper",
        id: "c-tp-3ply",
        brand: "Quilton",
        name: "3 Ply Toilet Rolls",
        size: "9 pack",
        price: dec!(13.00),
        was: None,
        unit: (dec!(0.40), "100ea"),
        deal: None,
        category: "Toilet Paper",
    },
    Row {
        store: Store::Woolworths,
        keyword: "banana",
        id: "w-banana",
        brand: "Woolworths",
        name: "Cavendish Bananas",
        size: "each",
        price: dec!(0.80),
        was: None,
        unit: (dec!(4.50), "1kg"),
        deal: None,
        category: "Fruit",
    },
    Row {
        store: Store::Coles,
        keyword: "banana",
        id: "c-banana",
        brand: "Coles",
        name: "Bananas",
        size: "approx. 170g",
        price: dec!(0.83),
        was: None,
        unit: (dec!(4.90), "1kg"),
        deal: None,
        category: "Fruit",
    },
];

/// A fake store answering from [`CATALOGUE`].
pub struct FakeStore {
    store: Store,
}

impl FakeStore {
    /// The fake for one store.
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Products of this store whose keyword appears in the query.
    fn matching(&self, query: &str) -> Result<Vec<Product>, StoreError> {
        let query = query.to_lowercase();
        if self.store == Store::Coles && query.contains(OUTAGE_KEYWORD) {
            return Err(StoreError::Unreachable("fake outage".into()));
        }
        Ok(CATALOGUE
            .iter()
            .filter(|row| row.store == self.store && query.contains(row.keyword))
            .map(product)
            .collect())
    }
}

/// A catalogue row as a [`Product`].
fn product(row: &Row) -> Product {
    Product {
        store: row.store,
        product_id: row.id.to_string(),
        name: row.name.to_string(),
        brand: Some(row.brand.to_string()),
        package_size: Some(row.size.to_string()),
        price: Some(row.price),
        was_price: row.was,
        on_special: row.was.is_some(),
        unit_price: measure::parse(row.unit.1).map(|m| normalise(row.unit.0, &m)),
        deals: row
            .deal
            .map(|(min_quantity, unit_price)| Deal {
                description: format!(
                    "{min_quantity} for ${}",
                    unit_price * Decimal::from(min_quantity)
                ),
                min_quantity,
                unit_price,
            })
            .into_iter()
            .collect(),
        category: Some(row.category.to_string()),
        url: format!("https://example.test/{}/{}", row.store, row.id),
        available: true,
    }
}

impl StoreClient for FakeStore {
    fn store(&self) -> Store {
        self.store
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(async move { self.matching(query) })
    }

    fn product<'a>(
        &'a self,
        product_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Product>, StoreError>> {
        Box::pin(async move {
            Ok(CATALOGUE
                .iter()
                .find(|row| row.store == self.store && row.id == product_id)
                .map(product))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn finds_products_by_keyword_for_its_own_store() {
        let found = FakeStore::new(Store::Coles)
            .search("toilet paper 3 ply")
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].store, Store::Coles);
    }

    #[tokio::test]
    async fn the_outage_keyword_fails_only_coles() {
        assert!(FakeStore::new(Store::Coles)
            .search("milk outage")
            .await
            .is_err());
        assert!(FakeStore::new(Store::Woolworths)
            .search("milk outage")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn looks_up_its_own_products_by_id() {
        let coles = FakeStore::new(Store::Coles);
        let id = coles.search("milk").await.unwrap()[0].product_id.clone();
        let found = coles.product(&id).await.unwrap().unwrap();
        assert_eq!(found.product_id, id);
        assert_eq!(
            FakeStore::new(Store::Woolworths)
                .product(&id)
                .await
                .unwrap(),
            None
        );
    }
}
