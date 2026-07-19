use arc_swap::ArcSwap;
use dashmap::DashMap;
use fastbloom::BloomFilter;

use crate::{document::entity::{CollectionID, DocumentID, DocumentSeqID, SegmentID}, shared::Document};

pub struct CollectionState {
    pub id: CollectionID,
    pub levels: ArcSwap<Vec<LevelState>>,
    pub buffer: ArcSwap<TransactionBuffer>,
    pub commit_buffer: ArcSwap<Option<TransactionBuffer>>,
}

impl CollectionState {
    pub fn doc_from_buffer_by_id(&self, document_id: &DocumentID) -> Option<Document> {
        let buffer = self.buffer.load();

        if let Some(doc) = buffer.id_to_seq_id.get(document_id).and_then(|seq_id| buffer.buffer.get(&seq_id)) {
            return Some(doc.clone());
        }

        let commit_buffer = self.commit_buffer.load();
        if let Some(cb) = commit_buffer.as_ref() {
            if let Some(doc) = cb.id_to_seq_id.get(document_id).and_then(|seq_id| cb.buffer.get(&seq_id)) {
                return Some(doc.clone());
            }
        }

        None
    }

    pub fn doc_from_buffer_by_seq_id(&self, seq_id: &DocumentSeqID) -> Option<Document> {
        let buffer = self.buffer.load();
        if let Some(doc) = buffer.buffer.get(seq_id) {
            return Some(doc.clone());
        };

        let commit_buffer = self.commit_buffer.load();
        if let Some(cb) = commit_buffer.as_ref() {
            if let Some(doc) = cb.buffer.get(seq_id) {
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

pub struct TransactionBuffer {
    pub id_to_seq_id: DashMap<DocumentID, DocumentSeqID>,
    pub buffer: DashMap<DocumentSeqID, Document>, // TODO: tombstone
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
    pub id_block_offsets: Vec<u64>,

    pub smallest_seq_id: DocumentSeqID,
    pub biggest_seq_id: DocumentSeqID,
    pub seq_id_filter: BloomFilter,
    pub seq_id_block_offsets: Vec<u64>,
}

impl SegmentState {
    pub fn may_contain_doc_id(&self, document_id: &DocumentID) -> bool {
        if document_id.0 < self.smallest_id.0 || document_id.0 > self.biggest_id.0 {
            return false;
        }

        self.id_filter.contains(&document_id.0)
    }

    pub fn may_contain_doc_seq_id(&self, seq_id: &DocumentSeqID) -> bool {
        if seq_id.0 < self.smallest_seq_id.0 || seq_id.0 > self.biggest_seq_id.0 {
            return false;
        }

        self.seq_id_filter.contains(&seq_id.0)
    }
}
