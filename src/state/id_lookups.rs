use crate::state::total_state::*;
use crate::version::*;

impl TotalState {
  pub fn luid_to_uuid(&self, luid: Luid) -> Option<Uuid> {
    self.universe.get_index(luid)
  }
  pub fn uuid_to_luid(&self, uuid: &Uuid) -> Option<Luid> {
    self.universe.get_index_of(uuid)
  }
}

impl Version {
  pub fn vlid_to_luid(&self, vlid: Vlid) -> Option<Luid> {
    self.version_universe.select(vlid)
  }
  pub fn luid_to_vlid(&self, luid: Luid) -> Option<Vlid> {
    match self.version_universe.contains(luid) {
      false => None,
      true => Some(self.version_universe.rank(luid)-1),
    }
  }
  pub fn slids0_to_vlid(&self, slid: Slid) -> Option<Vlid> {
    self.s0.select(slid)
  }
  pub fn vlid_to_slids0(&self, vlid: Vlid) -> Option<Slid> {
    match self.s0.contains(vlid) {
      false => None,
      true => Some(self.s0.rank(vlid)-1),
    }
  }
  pub fn slids0_of_vlid(&self, vlid: Vlid) -> Option<Slid> {
    for slids0 in 0..self.s0.len() {
      if self.s0i[slids0].contains(vlid) {
        return Some(slids0);
      }
    }
  }
  pub fn slid_to_vlid(&self, slids0: Slid, slid: Slid) -> Option<Vlid> {
    self.s0i[slids0 as usize].select(slid)
  }
  pub fn vlid_to_slid(&self, slids0: Slid, vlid: Vlid) -> Option<Slid> {
    match self.s0i[slids0 as usize].contains(vlid) {
      false => None,
      true => Some(self.s0i[slids0 as usize].rank(vlid)-1),
    }
  }
  pub fn vlid_to_slids0_and_slid(&self, vlid: Vlid) -> Option<(Slid, Slid)> {
    let slids0 = self.vlid_to_slids0(vlid)?;
    let slid = self.vlid_to_slid(slids0, vlid)?;
    Some((slids0, slid))
  }
  pub fn luid_to_slids0_and_slid(&self, luid: Luid) -> Option<(Slid, Slid)> {
    let vlid = self.luid_to_vlid(luid)?;
    self.vlid_to_slids0_and_slid(vlid)
  }
  pub fn uuid_to_slids0_and_slid(&self, uuid: &Uuid) -> Option<(Slid, Slid)> {
    let luid = self.uuid_to_luid(uuid)?;
    self.luid_to_slids0_and_slid(luid)
  }
  pub fn luid_to_slids0(&self, luid: Luid) -> Option<Slid> {
    let vlid = self.luid_to_vlid(luid)?;
    self.vlid_to_slids0(vlid)
  }
  pub fn uuid_to_slids0(&self, uuid: &Uuid) -> Option<Slid> {
    let luid = self.uuid_to_luid(uuid)?;
    self.luid_to_slids0(luid)
  }
}