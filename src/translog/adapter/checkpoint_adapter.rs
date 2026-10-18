use crate::translog::CheckpointPort;



pub struct CheckpointAdapter {

}

impl CheckpointAdapter {
    pub fn new() -> Self {
        CheckpointAdapter {
            
        }
    }
}

impl CheckpointPort for CheckpointAdapter {
    fn get_checkpoint_op_seq(&self, collection_id: &str) -> Result<u64, crate::utils::vixerr::Error> {
        todo!()
    }

    fn submit_checkpoint(&self, op_seq: u64) -> Result<(), crate::utils::vixerr::Error> {
        todo!()
    }
}