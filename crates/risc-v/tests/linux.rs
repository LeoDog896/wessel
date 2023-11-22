use risc_v::cpu::Xlen;
use risc_v::Emulator;
use risc_v::default_terminal::DefaultTerminal;
use risc_v::terminal::Terminal;

use std::fs;
use std::sync::{Arc, Mutex};

struct MultiThreadedTerminal {
    // input_buffer: Vec<u8>,
    // output_buffer: Vec<u8>,
    input_buffer: Arc<Mutex<Vec<u8>>>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
}

impl MultiThreadedTerminal {
    fn new() -> Self {
        Self {
            input_buffer: Arc::new(Mutex::new(Vec::new())),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Terminal for MultiThreadedTerminal {
    fn read(&mut self) -> u8 {
        let mut buffer = self.input_buffer.lock().unwrap();
        if buffer.len() == 0 {
            return 0;
        }
        let byte = buffer[0];
        buffer.remove(0);
        byte
    }

    fn write(&mut self, byte: u8) {
        let mut buffer = self.output_buffer.lock().unwrap();
        buffer.push(byte);
    }
}

#[test]
fn main() {
    let elf_contents = fs::read("resources/fw_payload.elf").unwrap();
    let fs_contents = fs::read("resources/rootfs.img").unwrap();

    let mut terminal = MultiThreadedTerminal::new();

    std::thread::spawn(move || {
        let mut emulator = Emulator::new(Box::new(terminal));
        emulator.setup_program(elf_contents);
    
        emulator.setup_filesystem(fs_contents);

        emulator.run();
    });
}
