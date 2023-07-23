use crate::context::*;
use roaring::RoaringTreemap;

pub struct Version {
  pub version_universe: RoaringTreemap, // of Luid
  pub s0: RoaringTreemap,               // of Vlid
  pub s0i: Vec<RoaringTreemap>,         // Slid(s0) -> Vlid
  pub ctx: Context,
}
