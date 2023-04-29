use std::rc::Rc;
use crate::interface::sound::resource::StaticSound;

// All sounds are internally represented as 48khz mono with 16 bits of depth. Blocks are BLOCK_LENGTH samples long. Default block length is 256.
pub enum BlockProvider<const BLOCK_SIZE: usize = 256> {
    Static {
        cursor: usize,
        data: Rc<StaticSound>
    }
}
impl<const SIZE: usize> BlockProvider<SIZE> {
    pub fn new_static(data: Rc<StaticSound>) -> Self {
        Self::Static {
            cursor: 0,
            data
        }
    }
}