//! Coles' search and product page data, as the website's Next.js data
//! routes return them.
//!
//! `GET /_next/data/<buildId>/en/search/products.json?q=<query>` answers
//! with the props the search page renders from. Only the fields this
//! application reads are declared. Field names are verbatim from a response
//! captured on 2026-10-01 and 2026-10-06 (see `tests/fixtures/coles/`).

use serde::Deserialize;

/// The data route's envelope.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub page_props: PageProps,
}

/// The search page's props.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
    pub search_results: Option<SearchResults>,
}

/// A product page's data route envelope
/// (`/_next/data/<buildId>/en/product/<slug>-<id>.json`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPage {
    pub page_props: ProductProps,
}

/// A product page's props: the product, or — when the slug was not the
/// real one — where Coles redirects to.
#[derive(Debug, Deserialize)]
pub struct ProductProps {
    pub product: Option<WireProduct>,
    #[serde(rename = "__N_REDIRECT")]
    pub redirect: Option<String>,
}

/// Results mix products with banner tiles, so each entry is read loosely
/// and only `"_type": "PRODUCT"` entries are decoded as products.
#[derive(Debug, Deserialize)]
pub struct SearchResults {
    #[serde(default)]
    pub results: Vec<serde_json::Value>,
}

/// One product as Coles describes it, in search results and on its page.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireProduct {
    pub id: i64,
    pub name: String,
    pub brand: Option<String>,
    pub size: Option<String>,
    #[serde(default)]
    pub availability: bool,
    /// Set on paid placements.
    pub ad_id: Option<serde_json::Value>,
    pub featured: Option<bool>,
    pub pricing: Option<Pricing>,
    #[serde(default)]
    pub online_heirs: Vec<Heir>,
}

/// Price data. Absent for products that cannot be bought.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pricing {
    pub now: Option<f64>,
    /// `0` when there is no previous price.
    pub was: Option<f64>,
    /// The unit price as shown, e.g. `$4.90/ 1kg`.
    pub comparable: Option<String>,
    /// `SPECIAL`, `EVERYDAY`, ...
    pub promotion_type: Option<String>,
    pub online_special: Option<bool>,
    pub multi_buy_promotion: Option<MultiBuy>,
    /// e.g. `Pick any 2 for $5.80`.
    pub offer_description: Option<String>,
}

/// A multibuy: `reward` is the per-unit price once `min_quantity` are bought.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiBuy {
    pub min_quantity: u32,
    pub reward: f64,
}

/// A place the product sits in the online aisles.
#[derive(Debug, Deserialize)]
pub struct Heir {
    pub category: Option<String>,
}
