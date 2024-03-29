use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use risc_v::terminal::Terminal;
use std::io::{self, stdout, Read};
use std::num::NonZeroU8;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::{str, thread};

/// Represents a terminal that can be used to interact with the emulator
/// using raw mode.
pub struct TTYTerminal {
    channel: Receiver<u8>,
}

impl TTYTerminal {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();

        let stdin_channel = spawn_stdin_channel();

        TTYTerminal {
            channel: stdin_channel,
        }
    }
}

impl Drop for TTYTerminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

/// Spawn a thread to read stdin and send it to a channel.
/// Since stdin().bytes() is blocking, we need to spawn a thread to read it.
/// This allows us to poll the channel instead of blocking on stdin.
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
    fn put_byte(&mut self, value: NonZeroU8) {
        let str = vec![value.get()];
        execute!(stdout(), Print(str::from_utf8(&str).unwrap())).unwrap();
    }

    fn get_input(&mut self) -> Option<NonZeroU8> {
        match self.channel.try_recv() {
            Ok(key) => NonZeroU8::new(key),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}
