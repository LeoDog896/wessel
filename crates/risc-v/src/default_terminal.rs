use crate::terminal::Terminal;

/// Standard `Terminal`.
pub struct DefaultTerminal {
    pub input_data: Vec<u8>,
    pub output_data: Vec<u8>,
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
    fn put_byte(&mut self, value: u8) {
        self.output_data.push(value);
    }

    fn get_input(&mut self) -> u8 {
        match !self.input_data.is_empty() {
            true => self.input_data.remove(0),
            false => 0,
        }
    }
}
