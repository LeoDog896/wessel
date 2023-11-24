use risc_v::terminal::Terminal;
use risc_v::Emulator;

use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

struct MultiThreadedTerminal {
    input_buffer: Arc<Mutex<Vec<NonZeroU8>>>,
    output_buffer: Arc<Mutex<Vec<NonZeroU8>>>,
}

fn str_to_vec(str: &str) -> Vec<NonZeroU8> {
    str.bytes().map(|b| NonZeroU8::new(b).unwrap()).collect()
}

fn is_sub<T: PartialEq>(haystack: &[T], needle: &[T], window: usize) -> bool {
    haystack
        .windows(needle.len())
        .rev()
        .take(window)
        .any(|c| c == needle)
}

fn wait_until(
    buffer: &Arc<Mutex<Vec<NonZeroU8>>>,
    values: Vec<NonZeroU8>,
    duration: Duration,
    window: usize,
) {
    let start = std::time::Instant::now();
    loop {
        let output_buffer = buffer.lock().unwrap();
        if is_sub(&output_buffer, &values, window) {
            break;
        }
        if start.elapsed() > duration {
            let mut buf_data = output_buffer
                .iter()
                .map(|v| v.get())
                .rev()
                .collect::<Vec<u8>>();
            buf_data.reverse();
            let data = std::str::from_utf8(&buf_data).unwrap();
            fs::write("error_log.txt", data).unwrap();
            panic!(
                "Timeout: {}\n\n{data}\n\n",
                std::str::from_utf8(&values.iter().map(|v| v.get()).collect::<Vec<u8>>()).unwrap(),
            );
        }
    }
}

fn write_to_buffer(buffer: &Arc<Mutex<Vec<NonZeroU8>>>, values: Vec<NonZeroU8>) {
    let mut output_buffer = buffer.lock().unwrap();
    output_buffer.extend(values);
}

impl Terminal for MultiThreadedTerminal {
    fn put_byte(&mut self, value: NonZeroU8) {
        self.output_buffer.lock().unwrap().push(value);
    }

    fn get_input(&mut self) -> Option<NonZeroU8> {
        self.input_buffer.lock().unwrap().pop()
    }
}

const CTRL_C: &str = "\u{3}";
const ENTER: &str = "\u{D}";

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
            input_buffer: local_input_buffer,
            output_buffer: local_output_buffer,
        }));
        emulator.setup_program(elf_contents);

        emulator.setup_filesystem(fs_contents);

        emulator.run();
    });

    wait_until(
        &output_buffer.clone(),
        str_to_vec("Please press Enter to activate this console"),
        Duration::from_secs(60),
        10,
    );

    let write_to_buffer = |value: &str| {
        write_to_buffer(&input_buffer.clone(), str_to_vec(value));
    };

    // TODO: wait with clock cycles instead of sleeping

    let typewriter = |value: &str, delay: Duration| {
        for c in value.chars() {
            write_to_buffer(&c.to_string());
            thread::sleep(delay);
        }
    };

    thread::sleep(Duration::from_millis(500));

    write_to_buffer("\n");

    wait_until(
        &output_buffer.clone(),
        str_to_vec("/ #"),
        Duration::from_secs(3),
        10,
    );

    // Check if LS works
    {
        thread::sleep(Duration::from_millis(500));

        typewriter("ls\n", Duration::from_millis(100));

        wait_until(
            &output_buffer.clone(),
            str_to_vec("lost+found"),
            Duration::from_secs(3),
            500,
        );
    }

    // Try using VI
    {
        thread::sleep(Duration::from_millis(500));

        typewriter(
            format!("vi my_file{ENTER}").as_str(),
            Duration::from_millis(200),
        );
        thread::sleep(Duration::from_millis(500));
        typewriter(
            format!("iHello {CTRL_C}iWorld!{CTRL_C}:wq{ENTER}cat my_file{ENTER}").as_str(),
            Duration::from_millis(200),
        );
        thread::sleep(Duration::from_millis(500));
        wait_until(
            &output_buffer.clone(),
            str_to_vec("World!Hello"),
            Duration::from_secs(3),
            50,
        );
    }
}
