use std::{
    fmt::{Debug, Display},
    io::Cursor,
    mem::size_of,
};

use super::{read_chunks, List, RiffChunk};
use crate::{
    omni::riff::{HumanBytes, OmniVersion, RiffChunkHeader},
    text::{
        Block, BlockType::*, Definition, Duration, LoopingMethod, PaletteManagement, RValue,
        Statement::*, ToBlock, Transparency,
    },
    types::Vec3,
};
use binrw::{binrw, prelude::*, NullString, VecArgs};
use modular_bitfield::prelude::*;

#[derive(Clone)]
pub struct ExtraString(Option<NullString>);

impl ExtraString {
    pub fn len(&self) -> usize {
        match &self.0 {
            Some(s) => s.len(),
            None => 0,
        }
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl Display for ExtraString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(s) => write!(f, "{}", s),
            None => write!(f, ""),
        }
    }
}

impl Debug for ExtraString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtraString(\"")?;
        match &self.0 {
            Some(s) => write!(f, "{}", s)?,
            None => write!(f, "")?,
        }
        write!(f, "\")")
    }
}

impl BinRead for ExtraString {
    type Args<'a> = VecArgs<()>;

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Self(if args.count < 1 {
            None
        } else {
            let v = Vec::<u8>::read_options(reader, endian, args)?;
            let mut cursor = Cursor::new(&v);
            let data = NullString::read_options(&mut cursor, endian, ())?;
            Some(data)
        }))
    }
}

impl BinWrite for ExtraString {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        match &self.0 {
            Some(s) => s.write_options(writer, endian, args),
            None => ().write_options(writer, endian, args),
        }
    }
}

#[bitfield]
#[binrw]
#[br(map(Self::from_bytes))]
#[derive(Debug, Clone)]
pub struct MxFlcFlags {
    has_palette_management: bool,
    unk0: B7,
    unk2: B24,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxFlcVideo {
    flags: MxFlcFlags,
    unk6: u32,
}

#[bitfield]
#[binrw]
#[br(map(Self::from_bytes))]
#[derive(Debug, Clone)]
pub struct MxSmkFlags {
    has_palette_management: bool,
    unk0: B7,
    unk2: B24,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxSmkVideo {
    flags: MxSmkFlags,
    unk6: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum MxVideoFileType {
    #[brw(magic(b" FLC"))]
    Flc(MxFlcVideo),
    #[brw(magic(b" SMK"))]
    Smk(MxSmkVideo),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxVideo {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
    filename: NullString,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    filetype: MxVideoFileType,
}

impl ToBlock for MxVideo {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![Assignment(
            "fileName".into(),
            RValue::String(self.filename.to_string()),
        )];
        if self.presenter != "".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }

        match &self.filetype {
            MxVideoFileType::Flc(f) => {
                if !f.flags.has_palette_management() {
                    statements.push(Assignment(
                        "paletteManagement".into(),
                        RValue::Definition(Definition::PaletteManagement(PaletteManagement::None)),
                    ))
                }
            }
            MxVideoFileType::Smk(s) => {
                if !s.flags.has_palette_management() {
                    statements.push(Assignment(
                        "paletteManagement".into(),
                        RValue::Definition(Definition::PaletteManagement(PaletteManagement::None)),
                    ))
                }
            }
        }

        if self.duration != 0 {
            statements.push(Assignment(
                "duration".into(),
                RValue::Definition(Definition::Duration(Duration(self.duration))),
            ))
        }
        if self.extra.is_some() {
            statements.push(Assignment(
                "extra".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: DefineAnim,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub enum MxSoundFileType {
    #[brw(magic(b" WAV"))]
    Wav(MxWavObject),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxSound {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
    filename: NullString,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    filetype: MxSoundFileType,
}

impl ToBlock for MxSound {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![Assignment(
            "fileName".into(),
            RValue::String(self.filename.to_string()),
        )];
        if self.presenter != "".into() && self.presenter != "Lego3DWavePresenter".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }

        let MxSoundFileType::Wav(wav) = &self.filetype;
        if wav.volume != 0x4F {
            statements.push(Assignment("volume".into(), RValue::Integer(wav.volume)))
        }

        if self.start_time != 0 {
            statements.push(Assignment(
                "startTime".into(),
                RValue::Integer(self.start_time),
            ))
        }
        if self.loops != 1 {
            statements.push(Assignment("loopCount".into(), RValue::Integer(self.loops)))
        }
        if !self.flags.no_loop() {
            statements.push(Assignment(
                "loopingMethod".into(),
                RValue::Definition(Definition::LoopingMethod(if self.flags.loop_cache() {
                    LoopingMethod::Cache
                } else if self.flags.loop_stream() {
                    LoopingMethod::Stream
                } else {
                    unreachable!()
                })),
            ))
        }
        if self.extra.is_some() {
            statements.push(Assignment(
                "entityName".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: DefineSound,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct MxWorld {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,

    #[br(magic(b"LIST"))]
    #[br(args(buf_size))]
    pub list: List,
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct MxPresenter {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,

    #[br(magic(b"LIST"))]
    #[br(args(buf_size))]
    pub list: List,
}

impl ToBlock for MxPresenter {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![];
        if self.presenter != "".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }
        if self.loops != 1 {
            statements.push(Assignment("loopCount".into(), RValue::Integer(self.loops)))
        }
        if !self.flags.no_loop() {
            statements.push(Assignment(
                "loopingMethod".into(),
                RValue::Definition(Definition::LoopingMethod(if self.flags.loop_cache() {
                    LoopingMethod::Cache
                } else if self.flags.loop_stream() {
                    LoopingMethod::Stream
                } else {
                    unreachable!()
                })),
            ))
        }
        
        let mut blocks_before = vec![];

        for chunk in &self.list.subchunks {
            statements.push(Declaration(chunk.get_name()));

            let (block, before, after) = chunk.to_block(false);
            blocks_before.extend(before);
            if let Some(b) = block {
                blocks_before.push(b);
            }
            blocks_before.extend(after);
        }

        if self.extra.is_some() {
            statements.push(Assignment(
                "extra".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: ParallelAction,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            blocks_before,
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxEvtEvent {
    unk5: u32,
    unk6: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum MxEventFileType {
    #[brw(magic(b" EVT"))]
    Evt(MxEvtEvent),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxEvent {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
    filename: NullString,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    filetype: MxEventFileType,
}

impl ToBlock for MxEvent {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![Assignment(
            "fileName".into(),
            RValue::String(
                self.filename
                    .to_string()
                    .trim_end_matches(".evt")
                    .to_string(),
            ),
        )];
        if self.presenter != "".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }
        if self.extra.is_some() {
            statements.push(Assignment(
                "extra".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: DefineEvent,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxAnimation {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum MxBitmapFileType {
    #[brw(magic(b" STL"))]
    Stl(MxStlObject),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxBitmap {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
    filename: NullString,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    filetype: MxBitmapFileType,
}

impl ToBlock for MxBitmap {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![Assignment(
            "fileName".into(),
            RValue::String(self.filename.to_string()),
        )];
        if self.presenter != "".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.duration != 0 {
            statements.push(Assignment(
                "duration".into(),
                RValue::Definition(Definition::Duration(Duration(self.duration))),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }

        let MxBitmapFileType::Stl(stl) = &self.filetype;
        if !stl.flags.has_palette_management() {
            statements.push(Assignment(
                "paletteManagement".into(),
                RValue::Definition(Definition::PaletteManagement(PaletteManagement::None)),
            ))
        }

        if self.flags.transparent() {
            statements.push(Assignment(
                "transparency".into(),
                RValue::Definition(Definition::Transparency(Transparency::Yes)),
            ))
        }

        if self.extra.is_some() {
            statements.push(Assignment(
                "extra".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: DefineStill,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxWavObject {
    unk5: u32,
    unk6: u32,
    volume: i32,
}

#[bitfield]
#[binrw]
#[br(map(Self::from_bytes))]
#[derive(Debug, Clone)]
pub struct MxStlFlags {
    has_palette_management: bool,
    unk0: B7,
    unk2: B24,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxStlObject {
    flags: MxStlFlags,
    unk6: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxObjObject {
    unk5: u32,
    unk6: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub enum MxObjectFileType {
    #[brw(magic(b" OBJ"))]
    Obj(MxObjObject),
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MxObject {
    presenter: NullString,
    unk0: u32,
    name: NullString,
    id: u32,
    flags: MxObFlags,
    start_time: i32,
    duration: i32,
    loops: i32,
    location: Vec3,
    direction: Vec3,
    up: Vec3,
    #[br(temp)]
    #[bw(try_calc(extra.len().try_into()))]
    extra_size: u16,
    #[br(count(extra_size as usize))]
    extra: ExtraString,
    filename: NullString,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    filetype: MxObjectFileType,
}

impl ToBlock for MxObject {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        let mut statements = vec![Assignment(
            "fileName".into(),
            RValue::String(self.filename.to_string()),
        )];
        if self.presenter != "".into() {
            statements.push(Assignment(
                "handlerClass".into(),
                RValue::String(self.presenter.to_string()),
            ))
        }
        if self.location != Vec3::ZERO {
            statements.push(Assignment("location".into(), RValue::Vec3(self.location)))
        }
        if self.direction != Vec3::Z {
            statements.push(Assignment("direction".into(), RValue::Vec3(self.direction)))
        }
        if self.up != Vec3::Y {
            statements.push(Assignment("up".into(), RValue::Vec3(self.up)))
        }
        if self.duration != 0 {
            statements.push(Assignment(
                "duration".into(),
                RValue::Definition(Definition::Duration(Duration(self.duration))),
            ))
        }
        if self.extra.is_some() {
            statements.push(Assignment(
                "extra".into(),
                RValue::String(self.extra.to_string()),
            ))
        }

        (
            Some(Block {
                id: self.id,
                block_type: DefineObject,
                name: self.name.to_string(),
                is_weave: top_level,
                statements,
            }),
            vec![],
            vec![],
        )
    }
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub enum MxObType {
    #[brw(magic(3u16))]
    Video(MxVideo),
    #[brw(magic(4u16))]
    Sound(MxSound),
    #[brw(magic(6u16))]
    World(#[br(args(buf_size))] MxWorld),
    #[brw(magic(7u16))]
    Presenter(#[br(args(buf_size))] MxPresenter),
    #[brw(magic(8u16))]
    Event(MxEvent),
    #[brw(magic(9u16))]
    Animation(MxAnimation),
    #[brw(magic(10u16))]
    Bitmap(MxBitmap),
    #[brw(magic(11u16))]
    Object(MxObject),
}

impl ToBlock for MxObType {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        match self {
            Self::Video(x) => x.to_block(top_level),
            Self::Sound(x) => x.to_block(top_level),
            Self::World(_) => todo!(),
            Self::Presenter(x) => x.to_block(top_level),
            Self::Event(x) => x.to_block(top_level),
            Self::Animation(_) => todo!(),
            Self::Bitmap(x) => x.to_block(top_level),
            Self::Object(x) => x.to_block(top_level),
        }
    }
}

impl MxObType {
    pub fn get_name(&self) -> String {
        match self {
            MxObType::Video(x) => x.name.to_string(),
            MxObType::Sound(x) => x.name.to_string(),
            MxObType::World(x) => x.name.to_string(),
            MxObType::Presenter(x) => x.name.to_string(),
            MxObType::Event(x) => x.name.to_string(),
            MxObType::Animation(x) => x.name.to_string(),
            MxObType::Bitmap(x) => x.name.to_string(),
            MxObType::Object(x) => x.name.to_string(),
        }
    }
}

#[bitfield]
#[binrw]
#[br(map(Self::from_bytes))]
#[derive(Debug, Clone)]
pub struct MxObFlags {
    loop_cache: bool,
    no_loop: bool,
    loop_stream: bool,
    transparent: bool,
    unk0: B1,
    unk1: bool,
    unk2: B2,
    unk3: B24,
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(buf_size: i32))]
pub struct MxOb {
    pub header: RiffChunkHeader,
    #[br(pad_size_to(header.size))]
    #[br(args(buf_size))]
    pub obj: MxObType,
}

impl ToBlock for MxOb {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>) {
        self.obj.to_block(top_level)
    }
}
