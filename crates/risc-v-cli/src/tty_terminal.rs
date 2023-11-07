use risc_v::terminal::Terminal;
use std::io::{Read, Write};
use std::str;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

/// Popup `Terminal` used for desktop program.
pub struct TTYTerminal {
    lock: std::io::StdoutLock<'static>,
}

impl TTYTerminal {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();

        let stdout = std::io::stdout();
        let lock = stdout.lock();

        TTYTerminal {
            lock,
        }
    }
}

impl Drop for TTYTerminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

impl Terminal for TTYTerminal {
    fn put_byte(&mut self, value: u8) {
        let str = vec![value];
        write!(self.lock, "{}", str::from_utf8(&str).unwrap()).unwrap();
    }

    fn get_input(&mut self) -> u8 {
        std::io::stdin()
            .bytes() 
            .next()
            .and_then(|result| result.ok())
            .unwrap()
    }

    // Wasm specific methods. No use.

    fn put_input(&mut self, _value: u8) {}

    fn get_output(&mut self) -> u8 {
        0 // dummy
    }
}
