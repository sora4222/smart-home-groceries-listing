//! Coles' products → [`Product`].

use super::wire::{MultiBuy, Pricing, SearchPage, WireProduct};
use crate::services::stores::deal::Deal;
use crate::services::stores::measure;
use crate::services::stores::money::{from_wire, positive_from_wire};
use crate::services::stores::product::Product;
use crate::services::stores::store::Store;
use crate::services::stores::unit_price::{normalise, UnitPrice};

/// The `_type` Coles gives product results.
const PRODUCT_TYPE: &str = "PRODUCT";

/// Every product on the page, in Coles' order, without paid placements.
///
/// A product entry that does not decode is skipped and logged rather than
/// failing the whole search.
pub fn products(page: SearchPage, site_url: &str) -> Vec<Product> {
    page.page_props
        .search_results
        .map(|results| results.results)
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| entry.get("_type").and_then(|t| t.as_str()) == Some(PRODUCT_TYPE))
        .filter_map(|entry| match serde_json::from_value::<WireProduct>(entry) {
            Ok(product) => Some(product),
            Err(err) => {
                tracing::warn!(store = "coles", %err, "skipping a product that did not decode");
                None
            }
        })
        .filter(|p| !is_advert(p))
        .map(|p| product(p, site_url))
        .collect()
}

/// A paid placement rather than a search result.
fn is_advert(p: &WireProduct) -> bool {
    let has_ad_id = p
        .ad_id
        .as_ref()
        .is_some_and(|id| !id.is_null() && id.as_str() != Some(""));
    has_ad_id || p.featured == Some(true)
}

/// One product.
fn product(p: WireProduct, site_url: &str) -> Product {
    let pricing = p.pricing.as_ref();
    let price = pricing.and_then(|pr| from_wire(pr.now));
    Product {
        store: Store::Coles,
        product_id: p.id.to_string(),
        url: product_url(site_url, &p),
        was_price: pricing
            .and_then(|pr| positive_from_wire(pr.was))
            .filter(|was| Some(*was) > price),
        on_special: pricing.is_some_and(is_special),
        unit_price: pricing.and_then(unit_price),
        deals: pricing.and_then(deal).into_iter().collect(),
        category: p
            .online_heirs
            .iter()
            .find_map(|heir| heir.category.clone())
            .filter(|c| !c.trim().is_empty()),
        available: p.availability && price.is_some(),
        name: p.name,
        brand: p.brand.filter(|b| !b.trim().is_empty()),
        package_size: p.size.filter(|s| !s.trim().is_empty()),
        price,
    }
}

/// Coles marks specials by promotion type, or flags an online-only one.
fn is_special(pricing: &Pricing) -> bool {
    pricing.promotion_type.as_deref() == Some("SPECIAL") || pricing.online_special == Some(true)
}

/// The normalised unit price, read from the comparable string.
///
/// The string is used rather than `unit.ofMeasure*`, which for weighed
/// produce says "per 1 g" beside a per-kilogram price.
fn unit_price(pricing: &Pricing) -> Option<UnitPrice> {
    let (price, measure) = measure::parse_comparable(pricing.comparable.as_deref()?)?;
    Some(normalise(price, &measure))
}

/// A multibuy deal. `reward` is already a per-unit price.
fn deal(pricing: &Pricing) -> Option<Deal> {
    let MultiBuy {
        min_quantity,
        reward,
    } = pricing.multi_buy_promotion.as_ref()?;
    if *min_quantity == 0 {
        return None;
    }
    let unit_price = from_wire(Some(*reward))?;
    let description = pricing
        .offer_description
        .clone()
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "{min_quantity} for ${}",
                unit_price * rust_decimal::Decimal::from(*min_quantity)
            )
        });
    Some(Deal {
        description,
        min_quantity: *min_quantity,
        unit_price,
    })
}

/// `https://www.coles.com.au/product/<brand-name-size>-<id>`, the shape of
/// Coles' own product links.
fn product_url(site_url: &str, p: &WireProduct) -> String {
    let words = [p.brand.as_deref(), Some(p.name.as_str()), p.size.as_deref()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let slug = slugify(&words);
    format!("{}/product/{slug}-{}", site_url.trim_end_matches('/'), p.id)
}

/// Lowercase words joined by single hyphens.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::unit_price::Basis;
    use rust_decimal_macros::dec;

    fn fixture() -> Vec<Product> {
        let json = include_str!("../../../../tests/fixtures/coles/search_soft_drink.json");
        products(
            serde_json::from_str(json).unwrap(),
            "https://www.coles.com.au",
        )
    }

    #[test]
    fn maps_a_product_with_a_multibuy() {
        let products = fixture();
        let coke = products.iter().find(|p| p.product_id == "123011").unwrap();
        assert_eq!(coke.brand.as_deref(), Some("Coca-Cola"));
        assert_eq!(coke.price, Some(dec!(4.5)));
        assert!(coke.on_special);
        assert_eq!(coke.category.as_deref(), Some("Soft Drinks"));
        assert_eq!(
            coke.url,
            "https://www.coles.com.au/product/coca-cola-classic-soft-drink-bottle-1-25l-123011"
        );
        assert_eq!(
            coke.deals,
            vec![Deal {
                description: "Pick any 2 for $5.80".into(),
                min_quantity: 2,
                unit_price: dec!(2.9),
            }]
        );
        let unit = coke.unit_price.as_ref().unwrap();
        assert_eq!(unit.amount, dec!(0.36));
        assert_eq!(unit.per, Basis::Per100Millilitres);
    }

    #[test]
    fn weighed_produce_reads_the_comparable_string() {
        let products = fixture();
        let bananas = products.iter().find(|p| p.product_id == "409499").unwrap();
        let unit = bananas.unit_price.as_ref().unwrap();
        assert_eq!(unit.amount, dec!(0.49));
        assert_eq!(unit.per, Basis::Per100Grams);
        assert_eq!(unit.converted_from.as_deref(), Some("1kg"));
        assert!(!bananas.on_special);
        assert_eq!(bananas.was_price, None);
    }

    #[test]
    fn a_special_keeps_its_previous_price() {
        let products = fixture();
        let lollies = products.iter().find(|p| p.product_id == "5556581").unwrap();
        assert!(lollies.on_special);
        assert_eq!(lollies.was_price, Some(dec!(5)));
    }

    #[test]
    fn tiles_adverts_and_unpriced_products_are_handled() {
        let products = fixture();
        assert_eq!(
            products.len(),
            4,
            "the banner tile and the advert are dropped"
        );
        let gone = products.iter().find(|p| p.product_id == "777").unwrap();
        assert!(!gone.available);
        assert_eq!(gone.price, None);
    }

    #[test]
    fn slugs_are_lowercase_hyphenated_words() {
        assert_eq!(
            slugify("Coles Full Cream Milk 3L"),
            "coles-full-cream-milk-3l"
        );
    }
}
