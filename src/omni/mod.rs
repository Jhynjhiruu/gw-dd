use self::riff::{ChunkId, List, MxHd, MxOf, RiffChunk, OMNI_ID, RIFF_ID};
use binrw::BinRead;
use std::io::{Read, Seek};
use thiserror::Error;

mod riff;

pub struct Omni {
    pub container_type: ChunkId,
    pub header: MxHd,
    pub offsets: MxOf,
    pub streams: List,
}

#[derive(Error, Debug)]
pub enum OmniParseError {
    #[error(transparent)]
    BinRW(#[from] binrw::Error),

    #[error("RIFF chunk not found at beginning of file")]
    NoRiffChunk,

    #[error("Not an Omni file (RIFF chunk type \"{0}\", expected \"OMNI\" or \"MxSt\")")]
    NotOmni(ChunkId),

    #[error("Unknown top-level chunk layout (expected a RIFF chunk with 3 children: MxHd, MxOf, LIST; try dumping the AST to inspect it)")]
    UnknownLayout,
}

pub type Result<T> = std::result::Result<T, OmniParseError>;

impl Omni {
    pub fn parse<T: Read + Seek>(stream: &mut T) -> Result<Self> {
        let riff_chunk = RiffChunk::read_args(stream, 0x10000)?;

        if !matches!(riff_chunk, RiffChunk::Riff(_)) {
            return Err(OmniParseError::NoRiffChunk);
        }

        let RiffChunk::Riff(root) = riff_chunk else {
            unreachable!()
        };

        /*if root.riff_type != OMNI_ID {
            return Err(OmniParseError::NotOmni(root.riff_type));
        }*/
        match root.riff_type {
            OMNI_ID => {}
            MXST_ID => {}
            _ => return Err(OmniParseError::NotOmni(root.riff_type)),
        }

        if root.subchunks.len() != 3 {
            return Err(OmniParseError::UnknownLayout);
        }

        let [RiffChunk::MxHd(header), RiffChunk::MxOf(offsets), RiffChunk::List(streams)]: [RiffChunk; 3] =
            root.subchunks.try_into().unwrap()
        else {
            return Err(OmniParseError::UnknownLayout);
        };

        Ok(Self {
            container_type: root.riff_type,
            header,
            offsets,
            streams,
        })
    }
}
