use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProductId(pub String);

impl ProductId {
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }

  pub fn generate() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }
}

impl From<String> for ProductId {
  fn from(id: String) -> Self {
    Self(id)
  }
}

impl std::fmt::Display for ProductId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
