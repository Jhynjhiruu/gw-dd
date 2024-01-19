use crate::text::{Block, BlockType::*, RValue, Statement::*, ToBlock};

use self::{mxob::MxOb, mxst::MxSt};
use binrw::{binrw, parser, BinRead, BinResult};
use bytes::HumanBytes;
use derivative::Derivative;
use modular_bitfield::prelude::*;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    io::SeekFrom::{Current, Start},
    mem::size_of,
};

mod bytes;
mod mxob;
mod mxst;

#[binrw]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ChunkId {
    pub value: [u8; 4],
}

pub const RIFF_ID: ChunkId = ChunkId { value: *b"RIFF" };
pub const OMNI_ID: ChunkId = ChunkId { value: *b"OMNI" };
pub const MXST_ID: ChunkId = ChunkId { value: *b"MxSt" };

impl Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.value))
    }
}

impl Debug for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct RiffChunkHeader {
    #[br(map(|x: u32| ((x + 1) & !1)))]
    pub size: u32,
}

#[binrw]
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct DummyRiffChunk {
    pub id: ChunkId,
    pub hdr: RiffChunkHeader,
    #[br(if(hdr.size >= 4))]
    pub sub_type: Option<ChunkId>,
    #[br(count(hdr.size - if sub_type.is_some() {4} else {0}))]
    #[derivative(Debug = "ignore")]
    pub data: Vec<u8>,
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct Riff {
    pub header: RiffChunkHeader,
    pub riff_type: ChunkId,
    #[br(parse_with(read_chunks))]
    #[br(args(header.size - 4, buf_size))]
    pub subchunks: Vec<RiffChunk>,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ActListCount {
    #[br(temp)]
    #[bw(try_calc(values.len().try_into()))]
    count: u32,
    #[br(count(count))]
    values: Vec<u16>,
}
#[binrw]
#[derive(Debug, Clone)]
pub struct RandListCount {
    rand_upper: u32,
    #[br(temp)]
    #[bw(try_calc(values.len().try_into()))]
    count: u32,
    #[br(count(count))]
    values: Vec<u16>,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum ListCount {
    #[brw(magic(b"Act\0"))]
    Act(ActListCount),
    #[brw(magic(b"RAND"))]
    Rand(u32, u32),
    Count(u32),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxChList {
    list_count: ListCount,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum LISTType {
    #[brw(magic(b"MxCh"))]
    MxCh(MxChList),
    Other(ChunkId),
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct List {
    pub header: RiffChunkHeader,
    pub list_type: LISTType,
    #[br(parse_with(read_chunks))]
    #[br(args(header.size - match &list_type { LISTType::MxCh(l) => { match l.list_count { ListCount::Act(_) => todo!(), ListCount::Rand(_, _) => 8, ListCount::Count(_) => 8 } }, LISTType::Other(_) => 4 }, buf_size))]
    pub subchunks: Vec<RiffChunk>,
}

#[binrw]
#[derive(Clone)]
pub struct OmniVersion {
    pub hi: u16,
    pub lo: u16,
}

impl Display for OmniVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}", self.hi, self.lo)
    }
}

impl Debug for OmniVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxHd {
    pub header: RiffChunkHeader,
    pub version: OmniVersion,
    pub buffer_size: HumanBytes<i32>,
    pub buffer_count: i32,
}

impl ToBlock for MxHd {
    fn to_block(&self, _: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        (
            Some(Block {
                id: u32::MAX,
                block_type: DefineSettings,
                name: "Configuration".into(),
                is_weave: false,
                statements: vec![
                    Assignment(
                        "bufferSizeKB".into(),
                        RValue::Integer(self.buffer_size.0 / 1024),
                    ),
                    Assignment("buffersNum".into(), RValue::Integer(self.buffer_count)),
                ],
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct MxOf {
    pub header: RiffChunkHeader,
    pub offset_count: u32,
    #[br(count((header.size as usize - 4)/size_of::<u32>()))]
    pub objects: Vec<u32>,
}

#[bitfield]
#[binrw]
#[br(map(Self::from_bytes))]
#[derive(Debug, Clone)]
#[repr(u16)]
pub struct MxChFlags {
    unk0: B1,
    end: bool,
    unk1: B2,
    split: bool,
    unk2: B3,
    unk3: B8,
}

#[binrw]
#[derive(Derivative, Clone)]
#[derivative(Debug)]
#[brw(little)]
pub struct MxCh {
    pub header: RiffChunkHeader,
    pub flags: MxChFlags,
    pub object: u32,
    pub time: u32,
    #[br(temp)]
    #[bw(try_calc((data.len() + if !data.is_empty() { 2 * size_of::<u32>() } else { 0 }).try_into()))]
    size: u32,
    #[br(count(header.size - 14))]
    #[derivative(Debug = "ignore")]
    pub data: Vec<u8>,
}

#[binrw]
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Pad {
    pub header: RiffChunkHeader,
    #[br(count(header.size))]
    #[derivative(Debug = "ignore")]
    pub data: Vec<u8>,
}

#[binrw]
#[derive(Debug, Clone)]
#[brw(little)]
#[br(import_raw(buf_size: i32))]
pub enum RiffChunk {
    #[br(magic(b"RIFF"))]
    Riff(#[br(args(buf_size))] Riff),

    #[br(magic(b"LIST"))]
    List(#[br(args(buf_size))] List),

    #[br(magic(b"MxHd"))]
    MxHd(MxHd),

    #[br(magic(b"MxOf"))]
    MxOf(MxOf),

    #[br(magic(b"MxCh"))]
    MxCh(MxCh),

    #[br(magic(b"MxOb"))]
    MxOb(#[br(args(buf_size))] Box<MxOb>),

    #[br(magic(b"MxSt"))]
    MxSt(#[br(args(buf_size))] Box<MxSt>),

    #[br(magic(b"pad "))]
    Pad(Pad),
    //Unknown(DummyRiffChunk),
}

impl RiffChunk {
    pub fn get_size(&self) -> u32 {
        match self {
            Self::Riff(x) => x.header.size,
            Self::List(x) => x.header.size,
            Self::MxHd(x) => x.header.size,
            Self::MxOf(x) => x.header.size,
            Self::MxCh(x) => x.header.size,
            Self::MxOb(x) => x.header.size,
            Self::MxSt(x) => x.header.size,
            Self::Pad(x) => x.header.size,
            //RiffChunk::Unknown(x) => x.hdr.size,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Self::Riff(x) => unreachable!(),
            Self::List(x) => unreachable!(),
            Self::MxHd(x) => unreachable!(),
            Self::MxOf(x) => unreachable!(),
            Self::MxCh(x) => unreachable!(),
            Self::MxOb(x) => x.obj.get_name(),
            Self::MxSt(x) => unreachable!(),
            Self::Pad(x) => unreachable!(),
            //RiffChunk::Unknown(x) => x.hdr.size,
        }
    }
}

impl ToBlock for RiffChunk {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        match self {
            Self::Riff(_) => todo!(),
            Self::List(_) => todo!(),
            Self::MxHd(x) => x.to_block(top_level),
            Self::MxOf(_) => todo!(),
            Self::MxCh(_) => todo!(),
            Self::MxOb(x) => x.to_block(top_level),
            Self::MxSt(x) => x.to_block(top_level),
            Self::Pad(_) => (None, vec![], vec![]),
        }
    }
}

#[parser(reader, endian)]
pub fn read_chunks(size: u32, mut buf_size: i32) -> BinResult<Vec<RiffChunk>> {
    let mut rv = vec![];

    let max_pos = reader.stream_position()? + size as u64;

    //println!("new max_pos: {:X}:{:X}", reader.stream_position()?, max_pos,);

    while reader.stream_position()? + ((size_of::<ChunkId>() + size_of::<RiffChunkHeader>()) as u64)
        < max_pos
    {
        //println!("\tchunk: {:X}", reader.stream_position()?);
        let before = reader.stream_position()?;

        let pos_in_buffer = before as i32 % buf_size;
        if pos_in_buffer + 8 > buf_size {
            reader.seek(Current((buf_size - pos_in_buffer) as i64))?;
            continue;
        }

        let chunk = RiffChunk::read_options(reader, endian, buf_size);
        /*if reader.stream_position()? % 2 != 0 && !packed {
            reader.seek(Current(1))?;
        }*/

        match chunk {
            Ok(c) => {
                //println!("{:?}", c);
                //println!("\t\tsize: {:X}", c.get_size());
                if reader.stream_position()? < before + c.get_size() as u64 + 8 {
                    /*println!(
                        "diff is {}",
                        before + c.get_size() as u64 + 8 - reader.stream_position()?
                    );*/
                    reader.seek(Start(before + c.get_size() as u64 + 8))?;
                }

                if let RiffChunk::MxHd(hd) = &c {
                    buf_size = hd.buffer_size.0
                }

                rv.push(c);
            }
            Err(e) if e.is_eof() => break,
            Err(e) => return Err(e),
        }
    }

    if reader.stream_position()? < max_pos {
        reader.seek(Start(max_pos))?;
    }

    /*if reader.stream_position()? % 2 != 0 {
        reader.seek(Current(1))?;
    }*/

    Ok(rv)
}
