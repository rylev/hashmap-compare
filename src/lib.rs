use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

type Bucket<K, V> = Vec<(K, V)>;
pub struct Basic<K: Hash + Eq, V> {
  buckets: Vec<Bucket<K, V>>,
  bucket_count: usize,
  item_count: usize,
  max_load_factor: f64,
  should_resize: bool,
}

const INITIAL_BUCKET_COUNT: usize = 4;

impl<K: Hash + Eq, V> Basic<K, V> {
  pub fn new() -> Basic<K, V> {
    Basic {
      buckets: Self::create_buckets(INITIAL_BUCKET_COUNT),
      bucket_count: INITIAL_BUCKET_COUNT,
      item_count: 0,
      max_load_factor: 0.6,
      should_resize: true,
    }
  }

  pub fn insert(&mut self, key: K, value: V) {
    let load_factor = self.item_count as f64 / self.bucket_count as f64;
    if self.should_resize && load_factor >= self.max_load_factor {
      self.resize();
    }
    let bucket_index = self.bucket_index(&key);
    let bucket = self.buckets.get_mut(bucket_index).unwrap();
    if let Some(i) = bucket.iter_mut().find(|(k, _)| k == &key) {
      *i = (key, value);
    } else {
      self.item_count += 1;
      bucket.push((key, value));
    }
  }

  pub fn get(&self, key: &K) -> Option<&V> {
    let bucket = self.buckets.get(self.bucket_index(&key))?;
    bucket.iter().find(|(k, _)| key == k).map(|(_, v)| v)
  }

  pub fn remove(&mut self, key: &K) -> Option<V> {
    let bucket_index = self.bucket_index(&key);
    let bucket = self.buckets.get_mut(bucket_index)?;
    let pos = bucket.iter().position(|(k, _)| k == key)?;
    let (_, v) = bucket.swap_remove(pos);
    Some(v)
  }

  fn resize(&mut self) {
    self.bucket_count = self.bucket_count * 2;
    let new_buckets = Self::create_buckets(self.bucket_count);
    let old_buckets = std::mem::replace(&mut self.buckets, new_buckets);
    for bucket in old_buckets.into_iter() {
      for (key, value) in bucket.into_iter() {
        let bucket_index = self.bucket_index(&key);
        let bucket = self.buckets.get_mut(bucket_index).unwrap();
        bucket.push((key, value))
      }
    }
  }

  fn bucket_index(&self, key: &K) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    (hash % self.bucket_count as u64) as usize
  }

  fn create_buckets(bucket_count: usize) -> Vec<Bucket<K, V>> {
    let mut buckets = Vec::with_capacity(bucket_count);
    for _ in 0..bucket_count {
      buckets.push(Vec::new());
    }
    buckets
  }
}

type Slot<K, V> = Option<((K, V), usize)>;
pub struct Advanced<K: Hash + Eq, V> {
  slots: Vec<Slot<K, V>>,
  slot_count: usize,
  item_count: usize,
  max_load_factor: f64,
  should_resize: bool,
}

impl<K: Hash + Eq, V> Advanced<K, V> {
  pub fn new() -> Advanced<K, V> {
    Advanced {
      slots: Self::create_slots(INITIAL_BUCKET_COUNT),
      slot_count: INITIAL_BUCKET_COUNT,
      item_count: 0,
      max_load_factor: 0.6,
      should_resize: true,
    }
  }

  pub fn insert(&mut self, key: K, value: V) {
    let load_factor = self.item_count as f64 / self.slot_count as f64;
    if self.should_resize && load_factor >= self.max_load_factor {
      self.resize();
    }
    let new_slot_index = self.slot_index(&key);
    if cfg!(not(feature = "robin")) {
      let slot = self.slot_mut(new_slot_index, &key).unwrap();
      let old = slot.replace(((key, value), new_slot_index));

      if old.is_none() {
        self.item_count += 1;
      }
    } else {
      let mut current_slot = ((key, value), new_slot_index);
      for (i, slot) in self.slots.iter_mut().enumerate().skip(new_slot_index) {
        match slot {
          Some(((k, _), slot_index)) => {
            let current_distance = i - *slot_index;
            let new_distance = i - current_slot.1;
            let current_key = &(current_slot.0).0;
            if current_key == k {
              *slot = Some(current_slot);
              return;
            } else if current_distance < new_distance {
              current_slot = slot.replace(current_slot).unwrap();
            }
          }
          None => {
            slot.replace(current_slot);
            return;
          }
        }
      }
      self.slots.push(Some(current_slot));
    }
  }

  pub fn get(&self, key: &K) -> Option<&V> {
    let slot_index = self.slot_index(key);
    let slot = self.slot(slot_index, key)?;
    match slot {
      Some(((_, ref v), _)) => Some(v),
      None => None,
    }
  }

  pub fn remove(&mut self, key: &K) -> Option<V> {
    let slot_index = self.slot_index(&key);
    let slot = self.slot_mut(slot_index, key)?;
    let ((_, v), _) = slot.take()?;
    Some(v)
  }

  fn resize(&mut self) {
    self.slot_count = self.slot_count * 2;
    let new_slots = Self::create_slots(self.slot_count);

    let old_slots = std::mem::replace(&mut self.slots, new_slots);
    for old_slot in old_slots.into_iter() {
      if let Some(((key, value), slot_index)) = old_slot {
        let slot = self.slot_mut(slot_index, &key).unwrap();
        *slot = Some(((key, value), slot_index));
      }
    }
  }

  fn slot_mut(&mut self, slot_index: usize, key: &K) -> Option<&mut Slot<K, V>> {
    self
      .slots
      .iter_mut()
      .skip(slot_index)
      .find(|item| match item {
        Some(((k, _), _)) => k == key,
        None => true,
      })
  }

  fn slot(&self, slot_index: usize, key: &K) -> Option<&Slot<K, V>> {
    self.slots.iter().skip(slot_index).find(|item| match item {
      Some(((k, _), _)) => k == key,
      None => true,
    })
  }

  fn slot_index(&self, key: &K) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    (hash % self.slot_count as u64) as usize
  }

  fn create_slots(slot_count: usize) -> Vec<Slot<K, V>> {
    let mut new_slots = Vec::with_capacity(slot_count);
    for _ in 0..slot_count {
      new_slots.push(None);
    }
    new_slots
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basic_works() {
    let mut basic = Basic::new();
    basic.insert("dude", "wow");
    basic.insert("foo", "bar");
    basic.insert("foo", "lol");

    assert_eq!(basic.get(&"dude"), Some(&"wow"));
    assert_eq!(basic.get(&"foo"), Some(&"lol"));
    assert_eq!(basic.get(&"foo"), Some(&"lol"));
    let removed = basic.remove(&"foo");
    assert_eq!(basic.get(&"foo"), None);
    assert_eq!(removed, Some("lol"));
    assert_eq!(basic.get(&"qux"), None);
  }

  #[test]
  fn advanced_works() {
    let mut advanced = Advanced::new();
    advanced.insert("dude", "wow");
    advanced.insert("foo", "bar");
    advanced.insert("foo", "lol");

    assert_eq!(advanced.get(&"dude"), Some(&"wow"));
    assert_eq!(advanced.get(&"foo"), Some(&"lol"));
    assert_eq!(advanced.get(&"foo"), Some(&"lol"));

    let removed = advanced.remove(&"foo");
    assert_eq!(advanced.get(&"foo"), None);
    assert_eq!(removed, Some("lol"));
    assert_eq!(advanced.get(&"qux"), None);
  }
}
