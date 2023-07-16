use base64ct::{Base64UrlUnpadded, Encoding};
pub use uuid::Uuid;

pub type Lid = usize;
pub type Luid = Lid;
pub type Vlid = Lid;
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
