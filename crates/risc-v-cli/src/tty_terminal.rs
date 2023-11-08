use risc_v::terminal::Terminal;
use std::io::{Read, Write, self};
use std::{str, thread};
use std::sync::mpsc::{Receiver, self, TryRecvError};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

/// Popup `Terminal` used for desktop program.
pub struct TTYTerminal {
    lock: std::io::StdoutLock<'static>,
    channel: Receiver<u8>
}

impl TTYTerminal {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();

        let stdin_channel = spawn_stdin_channel();
        let stdout = std::io::stdout();
        let lock = stdout.lock();

        TTYTerminal {
            lock,
            channel: stdin_channel
        }
    }
}

impl Drop for TTYTerminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

fn spawn_stdin_channel() -> Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
        let ch = io::stdin()
            .bytes() 
            .next()
            .and_then(|result| result.ok())
            .unwrap();
        tx.send(ch).unwrap();
    });
    rx
}

impl Terminal for TTYTerminal {
    fn put_byte(&mut self, value: u8) {
        let str = vec![value];
        write!(self.lock, "{}", str::from_utf8(&str).unwrap()).unwrap();
    }

    fn get_input(&mut self) -> u8 {
        return match self.channel.try_recv() {
            Ok(key) => key,
            Err(TryRecvError::Empty) => 0,
            Err(TryRecvError::Disconnected) => 0,
        }
    }

    // Wasm specific methods. No use.

    fn put_input(&mut self, _value: u8) {}

    fn get_output(&mut self) -> u8 {
        0 // dummy
    }
}
