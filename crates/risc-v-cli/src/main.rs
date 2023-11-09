mod tty_terminal;

use risc_v::cpu::Xlen;
use risc_v::Emulator;
use tty_terminal::TTYTerminal;

use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets bit mode. Default is auto detect from elf file
    #[clap(short, long, value_parser = valid_xlen)]
    xlen: Option<String>,

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

fn valid_xlen(s: &str) -> Result<u16, String> {
    let xlen: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid xlen (32, 64)"))?;
    if xlen == 32 || xlen == 64 {
        Ok(xlen as u16)
    } else {
        Err(format!("`{s}` isn't a valid xlen (32, 64)"))
    }
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let fs_contents = match cli.fs {
        Some(path) => {
            let mut file = File::open(path)?;
            let mut contents = vec![];
            file.read_to_end(&mut contents)?;
            contents
        }
        None => vec![],
    };

    let mut has_dtb = false;
    let dtb_contents = match cli.dtb {
        Some(path) => {
            has_dtb = true;
            let mut file = File::open(path)?;
            let mut contents = vec![];
            file.read_to_end(&mut contents)?;
            contents
        }
        None => vec![],
    };

    let elf_filename = cli.elf;
    let mut elf_file = File::open(elf_filename)?;
    let mut elf_contents = vec![];
    elf_file.read_to_end(&mut elf_contents)?;

    let mut emulator = Emulator::new(Box::new(TTYTerminal::new()));
    emulator.setup_program(elf_contents);

    if let Some(x) = cli.xlen {
        match x.as_str() {
            "32" => {
                println!("Force to 32-bit mode.");
                emulator.update_xlen(Xlen::Bit32);
            }
            "64" => {
                println!("Force to 64-bit mode.");
                emulator.update_xlen(Xlen::Bit64);
            }
            _ => unreachable!("Invalid xlen - this should be caught by clap"),
        }
    };

    emulator.setup_filesystem(fs_contents);
    if has_dtb {
        emulator.setup_dtb(dtb_contents);
    }
    if cli.page_cache {
        emulator.enable_page_cache(true);
    }
    emulator.run();
    Ok(())
}
