//! Finding the product a household member picked in an item's search.
//!
//! The request names only a store and the store's product id. Everything
//! saved about the product comes from the search answer this module looks
//! through, so a client cannot set its own price.

use crate::error::ApiError;
use crate::services::product_search::{ItemSearch, PricedProduct};
use crate::services::stores::Store;

/// The product `product_id` from `store`'s part of `search`.
///
/// Refused when the store could not be searched (503 — try again), when the
/// store no longer offers that product for this item and its chips (422), or
/// when the store cannot deliver it now (422).
pub fn find(search: ItemSearch, store: Store, product_id: &str) -> Result<PricedProduct, ApiError> {
    let item_name = search.item.name;
    let outcome = search
        .stores
        .into_iter()
        .find(|outcome| outcome.store == store)
        .ok_or_else(|| not_offered(store, &item_name))?;

    if let Some(err) = outcome.error {
        return Err(ApiError::ServiceUnavailable(
            err.user_message(store.display_name()),
        ));
    }

    let priced = outcome
        .products
        .into_iter()
        .find(|priced| priced.product.product_id == product_id)
        .ok_or_else(|| not_offered(store, &item_name))?;

    if !priced.product.available {
        return Err(ApiError::UnprocessableEntity(format!(
            "{} is unavailable at {} right now — choose another product.",
            priced.product.name,
            store.display_name()
        )));
    }
    Ok(priced)
}

/// The store's results for the item do not include the product.
fn not_offered(store: Store, item_name: &str) -> ApiError {
    ApiError::UnprocessableEntity(format!(
        "{} does not offer that product for {item_name} any more — compare prices again and \
         choose another.",
        store.display_name()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::db::{GroceryItem, GroceryItemSource, GroceryItemStatus};
    use crate::services::product_search::StoreOutcome;
    use crate::services::stores::product::fixtures::product;
    use crate::services::stores::StoreError;
    use rust_decimal_macros::dec;

    fn item() -> GroceryItem {
        GroceryItem {
            id: uuid::Uuid::nil(),
            name: "milk".into(),
            quantity: 1,
            status: GroceryItemStatus::Active,
            source: GroceryItemSource::Manual,
            note: None,
            filter_terms: Vec::new(),
            added_by_user_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    fn search(coles: StoreOutcome) -> ItemSearch {
        ItemSearch {
            item: item(),
            query: "milk".into(),
            stores: vec![coles],
        }
    }

    fn coles_with(products: Vec<crate::services::stores::Product>) -> StoreOutcome {
        StoreOutcome {
            store: Store::Coles,
            products: products
                .into_iter()
                .map(|p| PricedProduct::new(p, 1))
                .collect(),
            error: None,
        }
    }

    #[test]
    fn finds_the_named_product_at_the_named_store() {
        let found = find(
            search(coles_with(vec![product(Store::Coles, "Milk", dec!(4.95))])),
            Store::Coles,
            "milk",
        )
        .unwrap();
        assert_eq!(found.product.name, "Milk");
        assert_eq!(found.total_price, Some(dec!(4.95)));
    }

    #[test]
    fn a_product_id_from_another_store_is_not_offered() {
        let err = find(
            search(coles_with(vec![product(Store::Coles, "Milk", dec!(4.95))])),
            Store::Woolworths,
            "milk",
        )
        .unwrap_err();
        assert!(matches!(err, ApiError::UnprocessableEntity(_)), "{err:?}");
    }

    #[test]
    fn an_id_missing_from_the_results_is_not_offered() {
        let err = find(search(coles_with(Vec::new())), Store::Coles, "milk").unwrap_err();
        assert!(err.to_string().contains("does not offer"), "{err}");
    }

    #[test]
    fn an_unavailable_product_is_refused() {
        let mut gone = product(Store::Coles, "Milk", dec!(4.95));
        gone.available = false;
        let err = find(search(coles_with(vec![gone])), Store::Coles, "milk").unwrap_err();
        assert!(err.to_string().contains("unavailable"), "{err}");
    }

    #[test]
    fn a_store_that_failed_is_a_try_again() {
        let failed = StoreOutcome {
            store: Store::Coles,
            products: Vec::new(),
            error: Some(StoreError::Unreachable("timed out".into())),
        };
        let err = find(search(failed), Store::Coles, "milk").unwrap_err();
        assert!(matches!(err, ApiError::ServiceUnavailable(_)), "{err:?}");
    }
}
