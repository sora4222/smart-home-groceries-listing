//! Woolworths' products → [`Product`].

use rust_decimal::Decimal;

use super::wire::{Attributes, SearchResponse, Tag, WireProduct};
use crate::services::stores::deal::Deal;
use crate::services::stores::measure;
use crate::services::stores::money::{from_wire, positive_from_wire};
use crate::services::stores::product::Product;
use crate::services::stores::store::Store;
use crate::services::stores::unit_price::{normalise, UnitPrice};

/// Every product in a response, in Woolworths' order, without sponsored
/// placements — an advert is not a search result.
pub fn products(response: SearchResponse, site_url: &str) -> Vec<Product> {
    response
        .products
        .unwrap_or_default()
        .into_iter()
        .flat_map(|group| group.products)
        .filter(|p| p.is_sponsored_ad != Some(true))
        .map(|p| product(p, site_url))
        .collect()
}

/// One product.
fn product(p: WireProduct, site_url: &str) -> Product {
    let price = from_wire(p.price);
    Product {
        store: Store::Woolworths,
        product_id: p.stockcode.to_string(),
        url: product_url(site_url, p.stockcode, p.url_friendly_name.as_deref()),
        unit_price: unit_price(p.cup_price, p.cup_measure.as_deref()),
        was_price: positive_from_wire(p.was_price).filter(|was| Some(*was) > price),
        deals: p.centre_tag.as_ref().and_then(deal).into_iter().collect(),
        category: p.additional_attributes.as_ref().and_then(category),
        available: p.is_available && price.is_some(),
        on_special: p.is_on_special,
        name: p.name,
        brand: p.brand.filter(|b| !b.trim().is_empty()),
        package_size: p.package_size.filter(|s| !s.trim().is_empty()),
        price,
    }
}

/// The normalised unit price, when Woolworths gave one we can read.
fn unit_price(cup_price: Option<f64>, cup_measure: Option<&str>) -> Option<UnitPrice> {
    let price = from_wire(cup_price)?;
    let measure = measure::parse(cup_measure?)?;
    Some(normalise(price, &measure))
}

/// A multibuy, priced per unit inside a complete group.
fn deal(tag: &Tag) -> Option<Deal> {
    let multibuy = tag.multibuy_data.as_ref()?;
    if multibuy.quantity == 0 {
        return None;
    }
    let group_price = from_wire(Some(multibuy.price))?;
    let description = tag
        .tag_content_text
        .as_deref()
        .and_then(|text| text.split(" - ").next())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{} for ${group_price}", multibuy.quantity));
    Some(Deal {
        description,
        min_quantity: multibuy.quantity,
        unit_price: (group_price / Decimal::from(multibuy.quantity)).round_dp(4),
    })
}

/// The most specific shopper-facing category, else the internal one.
fn category(attributes: &Attributes) -> Option<String> {
    attributes
        .piescategorynamesjson
        .as_deref()
        .and_then(|json| serde_json::from_str::<Vec<String>>(json).ok())
        .and_then(|names| names.into_iter().last())
        .or_else(|| attributes.sapcategoryname.clone())
        .filter(|name| !name.trim().is_empty())
}

/// `https://www.woolworths.com.au/shop/productdetails/<stockcode>/<slug>`.
fn product_url(site_url: &str, stockcode: i64, slug: Option<&str>) -> String {
    let site = site_url.trim_end_matches('/');
    match slug.filter(|s| !s.is_empty()) {
        Some(slug) => format!("{site}/shop/productdetails/{stockcode}/{slug}"),
        None => format!("{site}/shop/productdetails/{stockcode}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stores::unit_price::Basis;
    use rust_decimal_macros::dec;

    fn fixture() -> Vec<Product> {
        let json = include_str!("../../../../tests/fixtures/woolworths/search_soft_drink.json");
        products(
            serde_json::from_str(json).unwrap(),
            "https://www.woolworths.com.au",
        )
    }

    #[test]
    fn maps_a_plain_product() {
        let products = fixture();
        let pepsi = products.iter().find(|p| p.product_id == "7985").unwrap();
        assert_eq!(pepsi.name, "Pepsi Max Cola No Sugar Soft Drink Bottle");
        assert_eq!(pepsi.package_size.as_deref(), Some("1.25L"));
        assert_eq!(pepsi.price, Some(dec!(4)));
        assert_eq!(pepsi.category.as_deref(), Some("Soft Drinks"));
        assert_eq!(
            pepsi.url,
            "https://www.woolworths.com.au/shop/productdetails/7985/pepsi-max-cola-no-sugar-soft-drink-bottle"
        );
        let unit = pepsi.unit_price.as_ref().unwrap();
        assert_eq!(unit.amount, dec!(0.32));
        assert_eq!(unit.per, Basis::Per100Millilitres);
        assert_eq!(unit.converted_from.as_deref(), Some("1L"));
    }

    #[test]
    fn reads_a_multibuy_as_a_per_unit_deal() {
        let products = fixture();
        let pepsi = products.iter().find(|p| p.product_id == "7985").unwrap();
        assert_eq!(
            pepsi.deals,
            vec![Deal {
                description: "2 for $6.50".into(),
                min_quantity: 2,
                unit_price: dec!(3.25),
            }]
        );
    }

    #[test]
    fn reads_a_special_and_its_previous_price() {
        let products = fixture();
        let ginger = products.iter().find(|p| p.product_id == "691996").unwrap();
        assert!(ginger.on_special);
        assert_eq!(ginger.was_price, Some(dec!(9.95)));
    }

    #[test]
    fn an_unpriced_product_is_unavailable_and_an_advert_is_dropped() {
        let products = fixture();
        let gone = products.iter().find(|p| p.product_id == "6088658").unwrap();
        assert!(!gone.available);
        assert_eq!(gone.price, None);
        assert_eq!(gone.unit_price, None);
        assert!(products.iter().all(|p| p.product_id != "999001"));
    }

    #[test]
    fn a_response_with_no_products_is_empty() {
        let empty: SearchResponse = serde_json::from_str(r#"{"Products":null}"#).unwrap();
        assert!(products(empty, "https://example.test").is_empty());
    }
}
