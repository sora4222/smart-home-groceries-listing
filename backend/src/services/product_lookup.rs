//! Looking up one product at one store by the store's own id.
//!
//! Used to check what a product costs now — for example a product the
//! household chose earlier — without searching by words. The store is
//! reached only through [`StoreClient`], so this works for any store in
//! [`StoreClients`].

use crate::error::ApiError;
use crate::services::stores::{Product, Store, StoreClient, StoreClients};

/// The product `product_id` at `store`.
///
/// 404 when the store does not sell it (or the id is not one of its ids);
/// 503 with the store's own sentence when the store could not answer.
pub async fn find(
    stores: &StoreClients,
    store: Store,
    product_id: &str,
) -> Result<Product, ApiError> {
    let client = client_for(stores, store)?;
    match client.product(product_id).await {
        Ok(Some(product)) => {
            tracing::info!(%store, product_id, "product looked up");
            Ok(product)
        }
        Ok(None) => {
            tracing::info!(%store, product_id, "product not sold by the store");
            Err(ApiError::NotFound(format!(
                "{} does not sell product {product_id}",
                store.display_name()
            )))
        }
        Err(err) => {
            tracing::warn!(%store, product_id, kind = err.kind(), %err, "product lookup failed");
            Err(ApiError::ServiceUnavailable(
                err.user_message(store.display_name()),
            ))
        }
    }
}

/// The configured client for `store`.
fn client_for(stores: &StoreClients, store: Store) -> Result<&dyn StoreClient, ApiError> {
    stores
        .iter()
        .find(|client| client.store() == store)
        .map(|client| client.as_ref())
        .ok_or_else(|| ApiError::Misconfigured(format!("no client for {store}")))
}
