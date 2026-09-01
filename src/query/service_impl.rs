use std::sync::{Arc, Mutex};

use crate::{collection::CollectionService, document::{Document, DocumentService}, query::{ParsedInsertParam, service::QueryService, service_param_result::InsertParam}, translog::TranslogService, utils::{counter::Counter, vixerr::Error, vixserial::Serial}};

pub struct QueryServiceImpl {
    collection_service: Arc<dyn CollectionService>,
    document_service: Arc<dyn DocumentService>,
    translog_service: Arc<dyn TranslogService>,
    global_counter: Arc<dyn Counter>,
    serial: Arc<Serial>,
}

impl QueryServiceImpl {
    pub fn new(
        collection_service: Arc<dyn CollectionService>,
        document_service: Arc<dyn DocumentService>,
        translog_service: Arc<dyn TranslogService>,
        global_counter: Arc<dyn Counter>,
    ) -> Arc<dyn QueryService> {
        let serial = Serial::new();
        let arc = Arc::new(QueryServiceImpl {
            collection_service,
            document_service,
            translog_service,
            global_counter,
            serial,
        });

        arc
    }
}

impl QueryService for QueryServiceImpl {
    fn get_by_id(&self, collection_id: &str, id: &str) -> Result<Document, Error> {
        self.document_service.get_by_id(collection_id, id)
    }

    fn insert(&self, param: InsertParam) -> Result<Document, Error> {
        let parsed_payload = self.collection_service.parse_raw_document_payload_by_id(&param.collection_id, &param.payload)?;
        let parsed_param = ParsedInsertParam {
            collection_id: param.collection_id,
            document_id: param.document_id,
            payload: parsed_payload,
        };

        let document_service = self.document_service.clone();
        let translog_service = self.translog_service.clone();
        let global_counter = self.global_counter.clone();
        let doc = self.serial.submit(Box::new(move || {
            let op_seq = global_counter.next();
            translog_service.insert(parsed_param.translog_insert_param(op_seq))?;

            let doc = document_service.insert(&parsed_param.collection_id.clone(), parsed_param.document_insert_param(op_seq))?;

            Ok(doc)
        }))?;

        Ok(doc)
    }
}
