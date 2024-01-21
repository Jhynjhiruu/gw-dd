use crate::{omni::Omni, types::Vec3};
use anyhow::{anyhow, Result};
use chumsky::Parser;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

mod parser;
mod preprocessor;

#[derive(Debug, Clone)]
pub enum LoopingMethod {
    Cache,
    None,
    Stream,
}

impl Display for LoopingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cache => "CACHE",
                Self::None => "NONE",
                Self::Stream => "STREAM",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Duration(pub i32);

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            -1 => write!(f, "INDEFINITE"),
            x => write!(f, "{x}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaletteManagement {
    None,
}

impl Display for PaletteManagement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "NONE",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Transparency {
    Yes,
    Fast,
}

impl Display for Transparency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Yes => "YES",
                Self::Fast => "FAST", // unknown purpose/encoding
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum Definition {
    LoopingMethod(LoopingMethod),
    Duration(Duration),
    PaletteManagement(PaletteManagement),
    Transparency(Transparency),
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoopingMethod(l) => write!(f, "{l}"),
            Self::Duration(d) => write!(f, "{d}"),
            Self::PaletteManagement(p) => write!(f, "{p}"),
            Self::Transparency(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub enum RValue {
    String(String),
    Integer(i32),
    Vec3(Vec3),
    Definition(Definition),
    Function(Function),
}

impl Display for RValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Vec3(v) => write!(f, "{v}"),
            Self::Definition(d) => write!(f, "{d}"),
            Self::Function(fun) => write!(f, "{fun}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(String, RValue),
    Declaration(String),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment(l, r) => write!(f, "{l} = {r}"),
            Self::Declaration(d) => write!(f, "{d}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    DefineSettings,
    DefineObject,
    DefineSound,
    DefineEvent,
    DefineAnim,
    ParallelAction,
    DefineStill,
    SerialAction,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DefineSettings => "defineSettings",
                Self::DefineObject => "defineObject",
                Self::DefineSound => "defineSound",
                Self::DefineEvent => "defineEvent",
                Self::DefineAnim => "defineAnim",
                Self::ParallelAction => "parallelAction",
                Self::DefineStill => "defineStill",
                Self::SerialAction => "serialAction",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub block_type: BlockType,
    pub name: String,
    pub is_weave: bool,
    pub statements: Vec<Statement>,
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}{} {{",
            self.block_type,
            self.name,
            if self.is_weave { " Weave" } else { "" }
        )?;
        for statement in &self.statements {
            writeln!(f, "\t{statement};")?;
        }
        writeln!(f, "}}\n")
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    elem: T,
    left: Option<Box<Tree<T>>>,
    right: Option<Box<Tree<T>>>,
}

impl<T: Clone> Tree<T> {
    pub fn new(elem: T) -> Self {
        Self {
            elem,
            left: None,
            right: None,
        }
    }

    pub fn add(elem: T) -> Box<Self> {
        Box::new(Self::new(elem))
    }

    pub fn insert_before(&mut self, elem: T) -> &mut Self {
        let insert = self.left.is_some();
        let left = self.left.get_or_insert(Self::add(elem.clone()));
        if insert {
            left.insert_before(elem)
        } else {
            left
        }
    }

    pub fn insert_after(&mut self, elem: T) -> &mut Self {
        let insert = self.right.is_some();
        let right = self.right.get_or_insert(Self::add(elem.clone()));
        if insert {
            right.insert_after(elem)
        } else {
            right
        }
    }

    pub fn insert_just_before(&mut self, elem: T) -> &mut Self {
        let insert = self.left.is_some();
        let left = self.left.get_or_insert(Self::add(elem.clone()));
        if insert {
            left.insert_after(elem)
        } else {
            left
        }
    }

    pub fn insert_just_after(&mut self, elem: T) -> &mut Self {
        let insert = self.right.is_some();
        let right = self.right.get_or_insert(Self::add(elem.clone()));
        if insert {
            right.insert_before(elem)
        } else {
            right
        }
    }

    pub fn traverse<F: FnMut(&T)>(&self, f: &mut F) {
        if let Some(l) = &self.left {
            l.traverse(f);
        }
        f(&self.elem);
        if let Some(r) = &self.right {
            r.traverse(f)
        }
    }
}

impl<T: Clone + Display> Tree<T> {
    pub fn collect(&self) -> impl Display {
        let mut rv = String::new();

        self.traverse(&mut |e: &T| rv += &e.to_string());

        rv
    }
}

#[derive(Debug)]
pub struct Text {
    settings: Block,
    blocks: BTreeMap<SortingId, Block>,
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.collect())
    }
}

pub trait ToBlock {
    fn to_block(&self, top_level: bool) -> (Option<Block>, Vec<Block>, Vec<Block>);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortingId {
    block_type: BlockType,
    id: u32,
    offset: u32,
    index: usize,
    parent_id: u32,
    parent_offset: u32,
    parent_index: usize,
}

impl PartialOrd for SortingId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortingId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.id.cmp(&other.id);

        if self.parent_id == other.id {
            return Ordering::Less;
        }

        if self.id == other.parent_id {
            return Ordering::Greater;
        }

        if self.offset != 0 && other.offset != 0 {
            return self.index.cmp(&other.index);
        }

        if self.offset == 0 && other.offset != 0 {
            return self.parent_id.cmp(&other.id);
        }

        if self.offset != 0 && other.offset == 0 {
            return self.id.cmp(&other.parent_id);
        }

        if self.parent_id != other.parent_id {
            //self.parent_id.cmp(&other.parent_id)
            return self.parent_offset.cmp(&other.parent_offset);
        }

        self.index.cmp(&other.index)
    }
}

impl SortingId {
    pub fn from_id_index(
        block_type: BlockType,
        id: u32,
        offsets: &[u32],
        index: usize,
        parent_id: u32,
        parent_index: usize,
    ) -> Self {
        Self {
            block_type,
            id,
            offset: *offsets.get(id as usize).unwrap_or(&0),
            index,
            parent_id,
            parent_offset: *offsets.get(parent_id as usize).unwrap_or(&0),
            parent_index,
        }
    }
}

impl Text {
    pub fn parse(file: &str) -> Result<Self> {
        let mut pp = preprocessor::Preprocessor::new();

        let file = pp.preprocess(file)?;

        println!("{file}");

        let (text, errs) = Self::parser().parse(&file).into_output_errors();

        text.ok_or(anyhow!("Parse error(s): {errs:?}"))
    }

    pub fn from_omni(omni: &Omni) -> Result<Self> {
        let (Some(settings), _, _) = omni.header.to_block(true) else {
            unreachable!()
        };

        //let mut blocks = Tree::new(settings);
        let mut blocks = BTreeMap::new();

        for (index, chunk) in omni.streams.subchunks.iter().enumerate() {
            let (block, blocks_before, blocks_after) = chunk.to_block(true);
            println!("{:?}", block);
            if let Some(b) = block {
                /*let cur = blocks.insert_after(b);
                for block in blocks_before {
                    cur.insert_just_before(block);
                }
                for block in blocks_after {
                    cur.insert_just_after(block);
                }*/

                let sorting_id = SortingId::from_id_index(
                    b.block_type,
                    b.id,
                    &omni.offsets.objects,
                    index,
                    b.id,
                    index,
                );

                let parent_id = b.id;
                println!("{:?}", sorting_id);
                println!("inserting: {:?}", blocks.insert(sorting_id, b));
                for (index_before, block_before) in blocks_before.into_iter().enumerate() {
                    println!("\tsub: {:?}", block_before);
                    let sorting_id_before = SortingId::from_id_index(
                        block_before.block_type,
                        block_before.id,
                        &omni.offsets.objects,
                        index_before,
                        parent_id,
                        index,
                    );
                    println!("\tsub: {:?}", sorting_id_before);
                    println!(
                        "\tinserting sub: {:?}",
                        blocks.insert(sorting_id_before, block_before)
                    );
                }
                for (index_after, block_after) in blocks_after.into_iter().enumerate() {
                    let sorting_id_after = SortingId::from_id_index(
                        block_after.block_type,
                        block_after.id,
                        &omni.offsets.objects,
                        index_after,
                        parent_id,
                        index,
                    );
                    println!(
                        "\tinserting sub: {:?}",
                        blocks.insert(sorting_id_after, block_after)
                    );
                }
            }
        }

        println!("{:#?}", blocks);

        Ok(Self { settings, blocks })
    }

    pub fn collect(&self) -> impl Display {
        let mut rv = self.settings.to_string();

        for block in self.blocks.values() {
            rv += &block.to_string();
        }

        rv
    }
}
