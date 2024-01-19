use anyhow::{anyhow, Result};
use clap::Parser;
use omni::Omni;
use std::{
    fs::{read, write},
    io::Cursor,
    path::PathBuf,
};
use text::Text;

mod omni;
mod text;
mod types;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    clap::ArgGroup::new("command").required(false)
))]
struct Args {
    /// Input file
    #[arg(short, long)]
    infile: PathBuf,

    /// Output file
    #[arg(short, long)]
    outfile: PathBuf,

    /// Resource folder
    #[arg(short, long)]
    resources: Option<PathBuf>,

    /// Prefix for stored paths (case-insensitive)
    #[arg(short, long)]
    prefix: Option<PathBuf>,

    /// Decompile given file
    #[arg(group = "command")]
    decompile: Option<bool>,

    /// Compile given file
    #[arg(group = "command")]
    compile: Option<bool>,

    /// Dump AST to file
    #[arg(long)]
    dump_ast: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let file = read(args.infile)?;
    let mut cursor = Cursor::new(&file);

    let omni = Omni::parse(&mut cursor)?;

    if let Some(path) = args.dump_ast {
        write(
            path,
            format!(
                "{:#?}\n\n({}) {:X?}\n\n{:#?}",
                omni.header,
                omni.offsets.objects.len(),
                omni.offsets,
                omni.streams
            ),
        )?;
    }

    let text = Text::from_omni(&omni)?;

    write(args.outfile, text.to_string())?;

    Ok(())
}
