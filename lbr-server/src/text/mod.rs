mod index;

use index::Index;

use crate::DocId;
use crate::Doc;

pub struct Indices {
    title: Index,
    authors: Index,
    keywords: Index,
}

impl Default for Indices {
    fn default() -> Self {
        Indices {
            title: Index::new(3),
            authors: Index::new(3),
            keywords: Index::new(3),
        }
    }
}
impl Indices {

    pub(crate) fn insert(&mut self, id: DocId, doc: &Doc) {
        self.title.insert(id, doc.title.to_lowercase().as_bytes());

        for author in &doc.authors {
            self.authors.insert(id, author.to_lowercase().as_bytes());
        }

        for keyword in &doc.keywords {
            self.keywords.insert(id, keyword.to_lowercase().as_bytes());
        }
    }

    pub fn remove(&mut self, id: DocId) {
        self.title.remove(id);
        self.authors.remove(id);
        self.keywords.remove(id);
    }

    pub fn search(&self, text: &str, limit: usize) -> Vec<(DocId, usize)> {
        let text = text.to_lowercase();
        let text = text.as_bytes();

        let mut scores = self.title.search(text);

        for (id, score) in self.authors.search(text) {
            *scores.entry(id).or_insert(0) += score;
        }

        for (id, score) in self.keywords.search(text) {
            *scores.entry(id).or_insert(0) += score;
        }

        let mut scores: Vec<_> = scores.into_iter().collect();
        scores.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));
        scores.truncate(limit);
        scores
    }
}
