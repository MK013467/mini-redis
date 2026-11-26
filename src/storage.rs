use std::collections::HashMap;

#[derive(Default)]
pub struct Storage {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl Storage {
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.map.get(key).map(Vec::as_slice)
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.map.insert(key, value);
    }

    pub fn del(&mut self, key: &[u8]) -> bool {
        self.map.remove(key).is_some()
    }
}
