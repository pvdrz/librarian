use std::collections::HashMap;

use crate::DocId;

pub struct Index {
    n: usize,
    grams: HashMap<Vec<u8>, Vec<DocId>>,
}

impl Index {
    pub fn new(n: usize) -> Self {
        Index {
            n,
            grams: Default::default(),
        }
    }

    pub fn search(&self, text: &[u8]) -> HashMap<DocId, usize> {
        let mut scores = HashMap::new();

        for gram in text.windows(self.n) {
            if let Some(ids) = self.grams.get(gram) {
                for &id in ids {
                    let score = scores.entry(id).or_insert(0);
                    *score += 1;
                }
            }
        }

        scores
    }

    pub fn insert(&mut self, id: DocId, text: &[u8]) {
        for gram in text.windows(self.n) {
            let ids = self
                .grams
                .entry(gram.to_vec())
                .or_insert_with(|| Vec::default());
            if let Err(pos) = ids.binary_search(&id) {
                ids.insert(pos, id);
            }
        }
    }

    pub fn remove(&mut self, id: DocId) {
        for ids in self.grams.values_mut() {
            if let Ok(pos) = ids.binary_search(&id) {
                ids.remove(pos);
            }
        }
    }
}

