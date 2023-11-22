use std::num::NonZeroU8;

use crate::terminal::Terminal;

/// Standard `Terminal`.
pub struct DefaultTerminal {
    pub input_data: Vec<NonZeroU8>,
    pub output_data: Vec<NonZeroU8>,
}

impl Default for DefaultTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTerminal {
    pub fn new() -> Self {
        Self {
            input_data: vec![],
            output_data: vec![],
        }
    }
}

impl Terminal for DefaultTerminal {
    fn put_byte(&mut self, value: NonZeroU8) {
        self.output_data.push(value);
    }

    fn get_input(&mut self) -> Option<NonZeroU8> {
        match !self.input_data.is_empty() {
            true => Some(self.input_data.remove(0)),
            false => None,
        }
    }
}
