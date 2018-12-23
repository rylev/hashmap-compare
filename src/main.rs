use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
fn main() {}

struct Basic<K: Hash + Eq, V>  {
  buckets: Vec<Vec<(K, V)>>,
  bucket_count: usize,
  item_count: usize,
  max_load_factor: f64,
  should_resize: bool
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
      bucket_count: INITIAL_BUCKET_COUNT,
      item_count: 0,
      max_load_factor: 0.6,
      should_resize: true,
    }
  }

  fn insert(&mut self, key: K, value: V) {
    let load_factor = self.item_count as f64 / self.bucket_count as f64;
    if self.should_resize && load_factor >= self.max_load_factor {
      self.resize();
    }
    let bucket_index = self.bucket_index(&key); 
    let bucket = self.buckets.get_mut(bucket_index).unwrap();
    if let Some(i) = bucket.iter_mut().find(|(k, _)| k == &key) {
      *i = (key, value);
    } else {
      self.item_count +=1;
      bucket.push((key, value));
    }
  }

  fn get(&self, key: K) -> Option<&V> {
    let bucket = self.buckets.get(self.bucket_index(&key))?;
    bucket.iter().find(|(k, _)| &key == k).map(|(_, v)| v)
  }

  fn resize(&mut self) {
    self.bucket_count = self.bucket_count * 2;
    let mut new_buckets = Vec::with_capacity(self.bucket_count);
    for _ in 0..self.bucket_count {
      new_buckets.push(Vec::new());
    }

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