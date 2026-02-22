#[allow(clippy::module_inception)]
pub mod event_publisher;
pub mod order_repository;
pub mod order_service;

#[cfg(test)]
mod order_service_test;
