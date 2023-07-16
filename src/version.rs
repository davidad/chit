use crate::id::*;
use crate::context::*;
use range_set_blaze::RangeSetBlaze;
use roaring::RoaringTreemap;

pub struct Version {
  pub version_universe: RangeSetBlaze<Luid>,
  pub s0: RoaringTreemap, // of Vlid
  pub s0i: Vec<RoaringTreemap>, // Slid(s0) -> Vlid
  pub ctx: Context,
}
