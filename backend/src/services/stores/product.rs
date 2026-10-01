//! A product as one store sells it, in the shape every store maps to.

use rust_decimal::Decimal;

use super::deal::Deal;
use super::store::Store;
use super::unit_price::UnitPrice;

/// One store's product, mapped from that store's own response.
///
/// Images and marketing copy are deliberately not carried: the household
/// compares on price, size and name, and the project aims to reduce
/// marketing-driven buying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub store: Store,
    /// The store's own id (Woolworths stockcode, Coles product id).
    pub product_id: String,
    pub name: String,
    pub brand: Option<String>,
    /// Pack size as the store writes it: `3L`, `approx. 170g`, `12 pack`.
    pub package_size: Option<String>,
    /// Shelf price for one unit. `None` when the store shows no price
    /// (usually because the product is unavailable).
    pub price: Option<Decimal>,
    /// The previous price when the product is reduced.
    pub was_price: Option<Decimal>,
    /// The store marks the product as on special.
    pub on_special: bool,
    pub unit_price: Option<UnitPrice>,
    pub deals: Vec<Deal>,
    /// The store's own category, used later for spending analysis.
    pub category: Option<String>,
    /// The product's page on the store's website.
    pub url: String,
    /// The store can deliver it now.
    pub available: bool,
}

impl Product {
    /// The words a filter chip is matched against: brand, name and size.
    pub fn searchable_text(&self) -> String {
        [
            self.brand.as_deref(),
            Some(self.name.as_str()),
            self.package_size.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! A product builder for tests elsewhere in the crate.

    use super::*;

    /// A plain, available product with a shelf price and nothing else.
    pub fn product(store: Store, name: &str, price: Decimal) -> Product {
        Product {
            store,
            product_id: name.to_lowercase().replace(' ', "-"),
            name: name.to_string(),
            brand: None,
            package_size: None,
            price: Some(price),
            was_price: None,
            on_special: false,
            unit_price: None,
            deals: Vec::new(),
            category: None,
            url: String::new(),
            available: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::product;
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn searchable_text_joins_brand_name_and_size() {
        let mut milk = product(Store::Coles, "Full Cream Milk", dec!(4.95));
        milk.brand = Some("Coles".into());
        milk.package_size = Some("3L".into());
        assert_eq!(milk.searchable_text(), "Coles Full Cream Milk 3L");
    }
}
