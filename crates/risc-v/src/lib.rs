// @TODO: temporal
const PROGRAM_MEMORY_CAPACITY: u64 = 1024 * 1024 * 128; // big enough to run Linux and xv6

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::num::NonZeroU8;

use elf::section::SectionHeader;
use fnv::{FnvHashMap, FnvHasher};

pub mod cpu;
pub mod default_terminal;
pub mod device;
pub mod memory;
pub mod mmu;
pub mod terminal;

use cpu::{Cpu, Xlen};
use terminal::Terminal;

use elf::endian::AnyEndian;
use elf::ElfBytes;

/// RISC-V emulator. It emulates RISC-V CPU and peripheral devices.
///
/// Sample code to run the emulator.
/// ```ignore
/// // Creates an emulator with arbitrary terminal
/// let mut emulator = Emulator::new(Box::new(DefaultTerminal::new()));
/// // Set up program content binary
/// emulator.setup_program(program_content);
/// // Set up Filesystem content binary
/// emulator.setup_filesystem(fs_content);
/// // Go!
/// emulator.run();
/// ```
pub struct Emulator {
    cpu: Cpu,

    /// Stores mapping from symbol to virtual address
    symbol_map: FnvHashMap<String, u64>,
}

impl Emulator {
    /// Creates a new `Emulator`. [`Terminal`]
    /// is internally used for transferring input/output data to/from `Emulator`.
    pub fn new(terminal: Box<dyn Terminal>) -> Self {
        Emulator {
            cpu: Cpu::new(terminal),

            symbol_map: FnvHashMap::default(),
        }
    }

    /// Runs program set by `setup_program()`.
    pub fn run(&mut self) {
        loop {
            self.tick();
        }
    }

    /// Helper method. Sends ascii code bytes to terminal.
    pub fn put_bytes_to_terminal(&mut self, bytes: &[NonZeroU8]) {
        for byte in bytes {
            self.cpu.get_mut_terminal().put_byte(*byte);
        }
    }

    /// Runs CPU one cycle
    pub fn tick(&mut self) {
        self.cpu.tick();
    }

    /// Sets up program run by the program. This method analyzes the passed content
    /// and configure CPU properly. If the passed contend doesn't seem ELF file,
    /// it panics. This method is expected to be called only once.
    ///
    /// # Arguments
    /// * `content` Program binary
    // @TODO: Make ElfAnalyzer and move the core logic there.
    // @TODO: Returns `Err` if the passed contend doesn't seem ELF file
    pub fn setup_program(&mut self, content: Vec<u8>) {
        let (analyzer, (program_data_section_headers, _, _)) =
            Self::load_program_for_symbols(&mut self.symbol_map, &content);

        // Detected whether the elf file is riscv-tests.
        // Setting up CPU and Memory depending on it.
        self.cpu.update_xlen(match analyzer.ehdr.class {
            elf::file::Class::ELF32 => Xlen::Bit32,
            elf::file::Class::ELF64 => Xlen::Bit64,
        });

        self.cpu.get_mut_mmu().init_memory(PROGRAM_MEMORY_CAPACITY);

        for section_header in program_data_section_headers {
            let sh_addr = section_header.sh_addr;
            let sh_offset = section_header.sh_offset as usize;
            let sh_size = section_header.sh_size as usize;
            if sh_addr >= 0x80000000 && sh_offset > 0 && sh_size > 0 {
                for j in 0..sh_size {
                    self.cpu
                        .get_mut_mmu()
                        .store_raw(sh_addr + j as u64, content[sh_offset + j]);
                }
            }
        }

        self.cpu.update_pc(analyzer.ehdr.e_entry);
    }

    /// Loads symbols of program and adds them to `symbol_map`.
    ///
    /// # Arguments
    /// * `content` Program binary
    pub fn load_program_for_symbols<'a>(
        symbol_map: &'a mut HashMap<String, u64, BuildHasherDefault<FnvHasher>>,
        content: &'a [u8],
    ) -> (
        ElfBytes<'a, AnyEndian>,
        (Vec<SectionHeader>, Vec<SectionHeader>, Vec<SectionHeader>),
    ) {
        let analyzer = ElfBytes::<AnyEndian>::minimal_parse(content)
            .expect("This file does not seem to be an ELF file");

        let section_headers = analyzer
            .section_headers()
            .expect("This file does not have section headers");

        let mut program_data_section_headers = vec![];
        let mut symbol_table_section_headers = vec![];
        let mut string_table_section_headers = vec![];

        for section_header in section_headers {
            match section_header.sh_type {
                1 => program_data_section_headers.push(section_header),
                2 => symbol_table_section_headers.push(section_header),
                3 => string_table_section_headers.push(section_header),
                _ => {}
            };
        }

        // Creates symbol - virtual address mapping
        if !string_table_section_headers.is_empty() {
            let (symbols, _) = analyzer.symbol_table().unwrap().unwrap();
            // Assuming symbols are in the first string table section.
            for symbol in symbols {
                symbol_map.insert(symbol.st_name.to_string(), symbol.st_value);
            }
        }

        (
            analyzer,
            (
                program_data_section_headers,
                symbol_table_section_headers,
                string_table_section_headers,
            ),
        )
    }

    /// Sets up filesystem. Use this method if program (e.g. Linux) uses
    /// filesystem. This method is expected to be called up to only once.
    ///
    /// # Arguments
    /// * `content` File system content binary
    pub fn setup_filesystem(&mut self, content: Vec<u8>) {
        self.cpu.get_mut_mmu().init_disk(content);
    }

    /// Sets up device tree. The emulator has default device tree configuration.
    /// If you want to override it, use this method. This method is expected to
    /// to be called up to only once.
    ///
    /// # Arguments
    /// * `content` DTB content binary
    pub fn setup_dtb(&mut self, content: Vec<u8>) {
        self.cpu.get_mut_mmu().init_dtb(content);
    }

    /// Updates XLEN (the width of an integer register in bits) in CPU.
    pub fn update_xlen(&mut self, xlen: Xlen) {
        self.cpu.update_xlen(xlen);
    }

    /// Enables or disables page cache optimization.
    /// Page cache optimization is experimental feature.
    /// See [`Mmu`] for the detail.
    pub fn enable_page_cache(&mut self, enabled: bool) {
        self.cpu.get_mut_mmu().enable_page_cache(enabled);
    }

    /// Returns mutable reference to [`Terminal`].
    pub fn get_mut_terminal(&mut self) -> &mut Box<dyn Terminal> {
        self.cpu.get_mut_terminal()
    }

    /// Returns immutable reference to [`Cpu`].
    pub fn get_cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Returns mutable reference to [`Cpu`].
    pub fn get_mut_cpu(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Returns a virtual address corresponding to symbol strings
    pub fn get_addredd_of_symbol(&self, symbol_string: &String) -> Option<u64> {
        self.symbol_map.get(symbol_string).copied()
    }
}

#[cfg(test)]
mod test_emulator {
    use super::*;
    use terminal::DummyTerminal;

    fn create_emu() -> Emulator {
        Emulator::new(Box::new(DummyTerminal::new()))
    }

    #[test]
    fn initialize() {
        let _emu = create_emu();
    }

    #[test]
    #[ignore]
    fn run() {}

    #[test]
    #[ignore]
    fn run_program() {}

    #[test]
    #[ignore]
    fn run_test() {}

    #[test]
    #[ignore]
    fn tick() {}

    #[test]
    #[ignore]
    fn setup_program() {}

    #[test]
    #[ignore]
    fn load_program_for_symbols() {}

    #[test]
    #[ignore]
    fn setup_filesystem() {}

    #[test]
    #[ignore]
    fn setup_dtb() {}

    #[test]
    #[ignore]
    fn update_xlen() {}

    #[test]
    #[ignore]
    fn enable_page_cache() {}

    #[test]
    #[ignore]
    fn get_addredd_of_symbol() {}
}
