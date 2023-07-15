use crate::id::*;
use range_set_blaze::RangeSetBlaze;

pub struct Version {
  pub local_universe: RangeSetBlaze<Luid>,
}
