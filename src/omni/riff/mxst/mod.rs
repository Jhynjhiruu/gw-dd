use std::mem::size_of;

use crate::{
    omni::riff::{HumanBytes, OmniVersion, RiffChunkHeader},
    text::{Block, BlockType::*, ToBlock},
};
use binrw::binrw;

use super::{
    mxob::{MxOb, MxObType::*},
    read_chunks, List, RiffChunk,
};

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct MxSt {
    pub header: RiffChunkHeader,
    #[br(magic(b"MxOb"))]
    #[br(args(buf_size))]
    pub obj: MxOb,
    #[br(magic(b"LIST"))]
    #[br(args(buf_size))]
    pub list: List,
}

impl ToBlock for MxSt {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        self.obj.to_block(top_level)
    }
}
