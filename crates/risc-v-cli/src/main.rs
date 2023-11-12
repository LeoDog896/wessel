mod tty_terminal;

use risc_v::cpu::Xlen;
use risc_v::Emulator;
use tty_terminal::TTYTerminal;

use std::fs;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets bit mode. Default is auto detected from the specified ELF file
    #[clap(value_enum, short, long)]
    xlen: Option<XLenArg>,

    /// File system image file
    #[clap(short, long)]
    fs: Option<String>,

    /// Device tree file
    #[clap(short, long)]
    dtb: Option<String>,

    /// Enable experimental page cache optimization
    #[clap(short, long)]
    page_cache: bool,

    /// The ELF file to run
    elf: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum XLenArg {
    /// Force 32-bit mode
    #[clap(name = "32")]
    Bit32,
    /// Force 64-bit mode
    #[clap(name = "64")]
    Bit64,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let fs_contents = cli.fs.map(fs::read).transpose()?.unwrap_or_default();
    let dtb_contents = cli.dtb.map(fs::read).transpose()?;
    let elf_contents = fs::read(cli.elf)?;

    let mut emulator = Emulator::new(Box::new(TTYTerminal::new()));
    emulator.setup_program(elf_contents);

    if let Some(x) = cli.xlen {
        match x {
            XLenArg::Bit32 => emulator.update_xlen(Xlen::Bit32),
            XLenArg::Bit64 => emulator.update_xlen(Xlen::Bit64),
        }
    };

    emulator.setup_filesystem(fs_contents);

    if let Some(dtb) = dtb_contents {
        emulator.setup_dtb(dtb);
    }

    if cli.page_cache {
        emulator.enable_page_cache(true);
    }
    emulator.run();
    Ok(())
}
