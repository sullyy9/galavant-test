use std::path::PathBuf;

use clap::Parser;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

#[derive(Parser, Debug, Clone, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub script: PathBuf,

    #[arg(short, long)]
    pub tcu: Option<String>,

    #[arg(short, long)]
    pub printer: Option<String>,

    #[arg(short, long)]
    pub debug: bool,
}

////////////////////////////////////////////////////////////////
