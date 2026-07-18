use arc_swap::ArcSwap;
use dashmap::DashMap;
use fastbloom::BloomFilter;

use crate::shared::Document;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectionID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SegmentID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentID(pub String);

pub struct CollectionState {
    pub id: CollectionID,
    pub levels: ArcSwap<Vec<LevelState>>,
    pub buffer: ArcSwap<DashMap<DocumentID, Document>>,
    pub commit_buffer: ArcSwap<Option<DashMap<DocumentID, Document>>>,
}

impl CollectionState {
    pub fn doc_from_buffer(&self, document_id: &DocumentID) -> Option<Document> {
        let buffer = self.buffer.load();
        if let Some(doc) = buffer.get(&document_id) {
            return Some(doc.clone());
        };

        let commit_buffer = self.commit_buffer.load();
        if let Some(cb) = commit_buffer.as_ref() {
            if let Some(doc) = cb.get(&document_id) {
                return Some(doc.clone());
            }
        }

        None
    }

    /// Returns a vector of all segment IDs in the collection, across all levels sorted from L0 to Ln.
    pub fn get_segment_ids(&self) -> Vec<SegmentID> {
        let levels = self.levels.load();
        levels.iter()
            .flat_map(|level| level.segment_ids.iter().cloned())
            .collect()
    }
}

pub struct LevelState {
    pub segment_ids: Vec<SegmentID>,
}

pub struct SegmentState {
    pub id: SegmentID,
    pub collection_id: CollectionID,

    pub smallest_id: DocumentID,
    pub biggest_id: DocumentID,

    pub id_filter: BloomFilter,
    pub seq_id_filter: BloomFilter,

    pub block_offsets: Vec<u64>,
}
