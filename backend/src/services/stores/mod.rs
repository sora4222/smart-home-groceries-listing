//! Store integrations: finding products at Woolworths and Coles, by search
//! words or by the store's own product id.
//!
//! Each store maps its own response into [`Product`], the one shape the rest
//! of the application reads. Prices are [`rust_decimal::Decimal`] from the
//! moment they arrive, unit prices are normalised to a common basis
//! ([`unit_price`]), and multibuy deals are parsed into [`Deal`]s whose
//! effect on an item's cost [`deal::total_price`] works out.
//!
//! This is the only module that makes HTTP calls to a store, and
//! [`registry`] is the only place a concrete client is named. See
//! `backend/skills/store-integration.md` for the anti-bot rules it follows.

pub mod cached;
pub mod client;
pub mod coles;
pub mod deal;
pub mod error;
pub mod fake;
pub mod http;
pub mod measure;
pub mod money;
pub mod product;
pub mod product_id;
pub mod registry;
pub mod store;
pub mod ttl_cache;
pub mod unit_price;
pub mod woolworths;

pub use client::{BoxFuture, StoreClient};
pub use deal::Deal;
pub use error::StoreError;
pub use product::Product;
pub use registry::StoreClients;
pub use store::Store;
pub use unit_price::{Basis, UnitPrice};
