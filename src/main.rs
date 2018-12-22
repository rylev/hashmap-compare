use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
fn main() {}

struct Basic<K: Hash + Eq, V>  {
  buckets: Vec<Vec<(K, V)>>,
  bucket_count: usize,
}

const INITIAL_BUCKET_COUNT: usize = 4;

impl <K: Hash+ Eq, V> Basic<K, V> {
  fn new() -> Basic<K, V> {
    let mut buckets = Vec::with_capacity(INITIAL_BUCKET_COUNT);
    for _ in 0..INITIAL_BUCKET_COUNT {
      buckets.push(Vec::new());
    }
    Basic {
      buckets,
      bucket_count: INITIAL_BUCKET_COUNT
    }
  }

  fn insert(&mut self, key: K, value: V) {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let bucket_index = (hash % self.bucket_count as u64) as usize;
    let bucket = self.buckets.get_mut(bucket_index).unwrap();
    if let Some(i) = bucket.iter_mut().find(|(k, _)| k == &key) {
      *i = (key, value);
    } else {
      bucket.push((key, value));
    }
  }

  fn get(&self, key: K) -> Option<&V> {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let bucket_index = (hash % self.bucket_count as u64) as usize;
    let bucket = self.buckets.get(bucket_index)?;
    bucket.iter().find(|(k, _)| &key == k).map(|(_, v)| v)
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
      basic.insert("foo", "lol");

      assert_eq!(basic.get("dude"), Some(&"wow"));
      assert_eq!(basic.get("foo"), Some(&"lol"));
      assert_eq!(basic.get("qux"), None);
    }
}