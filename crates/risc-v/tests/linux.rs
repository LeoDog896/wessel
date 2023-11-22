use risc_v::terminal::Terminal;
use risc_v::Emulator;

use std::fs;
use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};

struct MultiThreadedTerminal {
    input_buffer: Arc<Mutex<Vec<NonZeroU8>>>,
    output_buffer: Arc<Mutex<Vec<NonZeroU8>>>,
}

fn str_to_vec(str: &str) -> Vec<NonZeroU8> {
    str.bytes().map(|b| NonZeroU8::new(b).unwrap()).collect()
}

fn is_sub<T: PartialEq>(haystack: &[T], needle: &[T]) -> bool {
    haystack
        .windows(needle.len())
        .rev()
        .take(10)
        .any(|c| c == needle)
}

fn wait_until(buffer: &Arc<Mutex<Vec<NonZeroU8>>>, values: Vec<NonZeroU8>) {
    loop {
        let output_buffer = buffer.lock().unwrap();
        if is_sub(&output_buffer, &values) {
            break;
        }
    }
}

impl Terminal for MultiThreadedTerminal {
    fn put_byte(&mut self, value: NonZeroU8) {
        self.output_buffer.lock().unwrap().push(value);
    }

    fn get_input(&mut self) -> Option<NonZeroU8> {
        match !self.input_buffer.lock().unwrap().is_empty() {
            true => Some(self.input_buffer.lock().unwrap().remove(0)),
            false => None,
        }
    }
}

#[test]
fn main() {
    let elf_contents = fs::read("resources/fw_payload.elf").unwrap();
    let fs_contents = fs::read("resources/rootfs.img").unwrap();

    let input_buffer = Arc::new(Mutex::new(Vec::new()));
    let output_buffer = Arc::new(Mutex::new(Vec::new()));

    let local_input_buffer = input_buffer.clone();
    let local_output_buffer = output_buffer.clone();
    std::thread::spawn(move || {
        let mut emulator = Emulator::new(Box::new(MultiThreadedTerminal {
            input_buffer: local_input_buffer.clone(),
            output_buffer: local_output_buffer.clone(),
        }));
        emulator.setup_program(elf_contents);

        emulator.setup_filesystem(fs_contents);

        emulator.run();
    });

    wait_until(
        &output_buffer.clone(),
        str_to_vec("Please press Enter to activate this console."),
    );
    println!("Done booting...");
}
