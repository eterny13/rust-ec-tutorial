#[allow(clippy::module_inception)]
pub mod order;
pub mod order_id;
pub mod order_status;
pub mod order_error;
pub mod event;

pub use order::Order;
pub use order_error::OrderError;
pub use order_id::OrderId;
pub use order_status::OrderStatus;

#[cfg(test)]
mod order_test;
