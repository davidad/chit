use crate::id::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Context {
  Node(HashMap<String, Context>),
  Leaf(Luid),
}

impl Context {
  pub fn get(&self, mut path: impl Iterator<Item = impl ToString>) -> Option<Luid> {
    match self {
      Context::Leaf(luid) => match path.next() {
        None => Some(*luid),
        Some(_) => None,
      },
      Context::Node(map) => {
        let map = map;
        let key = path.next()?;
        let key = key.to_string();
        let context = map.get(&key)?;
        context.get(path)
      }
    }
  }
}

impl Default for Context {
  fn default() -> Self {
    Context::Node(HashMap::new())
  }
}
