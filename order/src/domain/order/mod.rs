#[allow(clippy::module_inception)]
pub mod order;
pub mod order_id;
pub mod order_status;
pub mod order_error;

pub use order::Order;
pub use order_id::OrderId;
pub use order_status::OrderStatus;
pub use order_error::OrderError;

#[cfg(test)]
mod order_test;
