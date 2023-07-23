use base64ct::{Base64UrlUnpadded, Encoding};

// Uuid = Universally Unique ID (128 bits; globally unique)
pub use uuid::Uuid;

// Local ID type (just an unsigned integer)
pub type Lid = usize;

// Luid = Locally Universal ID: local to an entire installation
pub type Luid = Lid;

// Vlid = Version-Local ID: local to a single version
pub type Vlid = Lid;

// Slid = Set-Local ID: local to a single finite set within a single version
pub type Slid = Lid;

pub trait AsBase64Url {
  fn as_base64url(&self) -> String;
}

impl AsBase64Url for Uuid {
  fn as_base64url(&self) -> String {
    Base64UrlUnpadded::encode_string(self.as_bytes())
  }
}

pub trait FromBase64Url {
  fn from_base64url(s: &str) -> Result<Self, String>
  where
    Self: Sized;
}

impl FromBase64Url for Uuid {
  fn from_base64url(s: &str) -> Result<Self, String> {
    let bytes = Base64UrlUnpadded::decode_vec(s)
      .map_err(|e| format!("Error decoding UUID in parsing base64url: {}", e))?;
    Uuid::from_slice(bytes.as_slice()).map_err(|e| format!("Error decoding UUID format: {}", e))
  }
}
