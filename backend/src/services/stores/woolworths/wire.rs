//! Woolworths' search response, as the website's own `Search/products` call
//! returns it.
//!
//! Only the fields this application reads are declared; serde ignores the
//! other hundred or so. Field names are verbatim from a response captured
//! on 2026-10-01 (see `tests/fixtures/woolworths/`).

use serde::Deserialize;

/// The request body the website sends. Kept to the fields the page itself
/// sends, so the call looks like the page's own.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchRequest<'a> {
    pub filters: [(); 0],
    pub is_special: bool,
    pub location: String,
    pub page_number: u32,
    pub page_size: u32,
    pub search_term: &'a str,
    pub sort_type: &'static str,
}

/// The top of the response. `Products` is `null` when nothing matched.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchResponse {
    #[serde(default)]
    pub products: Option<Vec<ProductGroup>>,
}

/// Woolworths groups variants of a product; each group holds one or more.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProductGroup {
    #[serde(default)]
    pub products: Vec<WireProduct>,
}

/// One product as Woolworths describes it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireProduct {
    pub stockcode: i64,
    pub name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub price: Option<f64>,
    pub was_price: Option<f64>,
    /// Unit price against `CupMeasure`, e.g. `3.13` per `100G`.
    pub cup_price: Option<f64>,
    pub cup_measure: Option<String>,
    #[serde(default)]
    pub is_on_special: bool,
    #[serde(default)]
    pub is_available: bool,
    #[serde(default)]
    pub is_sponsored_ad: Option<bool>,
    pub url_friendly_name: Option<String>,
    pub centre_tag: Option<Tag>,
    pub additional_attributes: Option<Attributes>,
}

/// A promotional tag shown on the product tile; multibuys live here.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub multibuy_data: Option<Multibuy>,
    /// Plain text of the tag, e.g. `2 for $6.00 - $1.88/100G `.
    pub tag_content_text: Option<String>,
}

/// "`Quantity` for `$Price`".
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Multibuy {
    pub quantity: u32,
    /// The price of the whole group, not of one unit.
    pub price: f64,
}

/// Category data. These keys are lowercase in the response.
#[derive(Debug, Deserialize)]
pub struct Attributes {
    /// A JSON-encoded array of category names, most specific last.
    pub piescategorynamesjson: Option<String>,
    /// The internal category, e.g. `DAIRY - MILK`.
    pub sapcategoryname: Option<String>,
}
