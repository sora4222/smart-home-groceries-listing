//! A purchase builder for tests in this crate.

use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::db::{CategorySource, GroceryItemStatus, Purchase};
use crate::services::stores::Store;

/// One of `product_id` at `store` for `price`, bought at noon UTC on `day`
/// (`YYYY-MM-DD`), for the item "milk" in "Dairy & eggs", no delivery share.
pub fn purchase(store: Store, product_id: &str, price: Decimal, day: &str) -> Purchase {
    let date = NaiveDate::parse_from_str(day, "%Y-%m-%d").expect("a YYYY-MM-DD day");
    Purchase {
        id: Uuid::new_v4(),
        order_id: Uuid::new_v4(),
        grocery_item_id: None,
        item_name: "milk".into(),
        item_key: "milk".into(),
        item_status_before: GroceryItemStatus::Committed,
        store,
        product_id: product_id.into(),
        product_name: product_id.into(),
        brand: None,
        package_size: None,
        quantity: 1,
        unit_price: price,
        shelf_price: Some(price),
        total_price: price,
        delivery_fee_share: Decimal::ZERO,
        store_category: None,
        category: "Dairy & eggs".into(),
        category_source: CategorySource::Name,
        bought_at: Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0).expect("noon")),
    }
}
