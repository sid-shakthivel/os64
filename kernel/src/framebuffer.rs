// src/framebuffer.rs

/*
    Framebuffer is portion of RAM which contains a bitmap which maps to the display
    GRUB sets the correct video mode before loading the kernel as specified within the multiboot header
    Pitch is number of bytes per row, BPP is bit depth
*/

/* 
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
*/

use lazy_static::lazy_static;
use core::fmt;
use multiboot2::{FramebufferTag};

use crate::page_frame_allocator::{self, PageFrameAllocator};
use crate::paging;

const SCREEN_WIDTH: usize = 1024;
const SCREEN_HEIGHT: usize = 768;

pub struct Framebuffer {
    terminal_row: usize,
    terminal_col: usize,
    framebuffer: &'static mut [u32; SCREEN_HEIGHT * SCREEN_WIDTH], 
    pitch: usize,
    bpp: usize,
    font: Option<PsfFont>,
    font_start: u32
}

#[derive(Debug, Clone, Copy)]
struct PsfFont {
    magic: u32,
    version: u32, // Usually 0
    header_size: u32, // Offset of bitmaps
    flags: u32,
    glymph_num: u32, 
    bytes_per_glyph: u32, // Size
    height: u32, // In pixels
    width: u32 // In pixels
}

lazy_static! {
    pub static ref TERMINAL: spin::Mutex<Framebuffer> = spin::Mutex::new(Framebuffer {
        terminal_row: 0,
        terminal_col: 0,
        framebuffer: unsafe { &mut *(0x180000 as *mut [u32; SCREEN_HEIGHT * SCREEN_WIDTH]) },
        pitch: 0,
        bpp: 0,
        font: None,
        font_start: 0,
    });
}

pub fn init(framebuffer_tag: FramebufferTag, page_frame_allocator: &mut PageFrameAllocator) {

    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let font = unsafe { &*(font_start as *const PsfFont) };

    TERMINAL.lock().pitch = framebuffer_tag.pitch as usize;
    TERMINAL.lock().bpp = framebuffer_tag.bpp as usize;
    TERMINAL.lock().font = Some(*font);
    TERMINAL.lock().font_start = font_start;

    paging::identity_map_from(framebuffer_tag.address, 3, page_frame_allocator);
}

#[macro_export] 
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        TERMINAL.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

impl fmt::Write for Framebuffer {
     // To support the rust formatting system and use the write! macro, the write_str method must be supported
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Framebuffer {
    pub fn write_string(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
    }

    fn put_char(&mut self, character: char) {
        // Each glyph is stored in a bitmap of 8 * 16

        match character {
            '\n' => self.new_line(),
            _ => self.draw_char(character),
        }
    }

    fn draw_char(&mut self, character: char) {
        let font = self.font.unwrap();
        let test = self.font_start + font.header_size + (font.bytes_per_glyph * (character as u32));

        let glyph_address = test as *mut u8;
        
        for cy in 0..16 {
            let mut index = 8;
            for cx in (0..8) {
                // Load correct bitmap for glyph
                let glyph_offset: u16 = unsafe { (*glyph_address.offset(cy) as u16) & (1 << index) };
                if glyph_offset > 0 {
                    self.draw_pixel(cx + self.terminal_col, cy as usize + self.terminal_row, 0xFFFFFFFF);
                } else {
                    self.draw_pixel(cx + self.terminal_col, cy as usize + self.terminal_row, 0x00);
                }
                index -= 1;
            }
        }

        self.terminal_col += 8;
        if (self.terminal_col >= SCREEN_WIDTH) { self.new_line(); }
    }

    fn draw_pixel(&mut self, x: usize, y: usize, byte: u32) {
        unsafe {
            let offset = (0x180000 + (y * self.pitch) + ((x * self.bpp) / 8)) as *mut u32;
            *offset = byte;
        }
    }

    fn new_line(&mut self) {
        self.terminal_col = 0;
        self.terminal_row += 16;
        // if (self.terminal_row >= SCREEN_HEIGHT) { self.scroll(); }
    }
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}