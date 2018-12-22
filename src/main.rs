use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
fn main() {}

struct Basic<K: Hash + Eq, V>  {
  buckets: Vec<Vec<(K, V)>>
}

const BUCKET_COUNT: usize = 4;

impl <K: Hash+ Eq, V> Basic<K, V> {
  fn new() -> Basic<K, V> {
    let mut buckets = Vec::with_capacity(BUCKET_COUNT);
    for _ in 0..BUCKET_COUNT {
      buckets.push(Vec::new());
    }
    Basic {
      buckets
    }
  }

  fn insert(&mut self, key: K, value: V) {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let bucket_index = (hash % BUCKET_COUNT as u64) as usize;
    let bucket = self.buckets.get_mut(bucket_index).unwrap();
    bucket.push((key, value));
  }

  fn get(&self, key: K) -> Option<&V> {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let bucket_index = (hash % BUCKET_COUNT as u64) as usize;
    let bucket = self.buckets.get(bucket_index)?;
    bucket.iter().find(|(k, _)| {
      &key == k
    }).map(|(_, v)| v)
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_insert_and_get() {
      let mut basic = Basic::new();
      basic.insert("dude", "wow");
      basic.insert("foo", "bar");

      assert_eq!(basic.get("dude"), Some(&"wow"));
      assert_eq!(basic.get("foo"), Some(&"bar"));
      assert_eq!(basic.get("qux"), None);
    }
}