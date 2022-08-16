// src/writer.rs

// This trait will be used for framebuffer, vga_text, uart as they all are used for outputting data to the screen

pub trait Writer {
    fn write_string(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
    }

    fn put_char(&mut self, character: char) {}
    fn newline(&mut self) {}
    fn clear(&mut self) {}
}
