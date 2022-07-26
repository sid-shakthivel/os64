// src/framebuffer.rs

/*
    Framebuffer is portion of RAM which contains a bitmap which maps to the display (pixels)
    GRUB sets the correct video mode before loading the kernel as specified within the multiboot header
    Pitch is number of bytes per row, BPP is bit depth
*/

/* 
    PSF(PC Screen Font) fonts consist of header, font, and unicode information 
    Glyphs are bitmaps of 8*16
*/

use lazy_static::lazy_static;
use core::fmt;
use multiboot2::{FramebufferTag};

use crate::page_frame_allocator::{self, PageFrameAllocator};
use crate::paging;

const SCREEN_WIDTH: u64 = 1024;
const SCREEN_HEIGHT: u64 = 768;

pub struct Framebuffer {
    framebuffer: &'static mut [u32; 786432], 
    pitch: u64,
    bpp: u64,
    colour_num: usize
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

pub struct Window {
    x: u64,
    y: u64,
    z: u64,
    width: u64,
    height: u64,
    next: Option<*mut Window>,
    prev: Option<*mut Window>,
}

impl Window {
    pub const fn new(x: u64, y: u64, width: u64, height: u64) -> Window {
        return Window {
            x: x,
            y: x,
            z: 0,
            width: width,
            height: height,
            next: None,
            prev: None,
        }
    }

    pub fn paint(&self) {
        FRAMEBUFFER.lock().fill_rect(self.x, self.y, self.width, self.height);
    }
}

lazy_static! {
    pub static ref FRAMEBUFFER: spin::Mutex<Framebuffer> = spin::Mutex::new(Framebuffer {
        framebuffer: unsafe { &mut *(0x180000 as *mut [u32; 786432]) },
        pitch: 0,
        bpp: 0,
        colour_num: 0,
    });
}

pub fn init(framebuffer_tag: FramebufferTag, page_frame_allocator: &mut PageFrameAllocator) {

    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let font = unsafe { &*(font_start as *const PsfFont) };

    FRAMEBUFFER.lock().pitch = framebuffer_tag.pitch as u64;
    FRAMEBUFFER.lock().bpp = framebuffer_tag.bpp as u64;

    paging::identity_map_from(framebuffer_tag.address, 3, page_frame_allocator);
}

impl Framebuffer {
    pub fn write_string(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
    }

    fn put_char(&mut self, character: char) {
        match character {
            _ => self.draw_char(character),
        }
    }

    fn draw_char(&mut self, character: char) {
        // let font = self.font.unwrap();
        // let glyph_address = (self.font_start + font.header_size + (font.bytes_per_glyph * (character as u32))) as *mut u8;
        
        // for cy in 0..16 {
        //     let mut index = 8;
        //     for cx in (0..8) {
        //         // Load correct bitmap for glyph
        //         let glyph_offset: u16 = unsafe { (*glyph_address.offset(cy) as u16) & (1 << index) };
        //         if glyph_offset > 0 {
        //             self.draw_pixel(cx + self.terminal_col, cy as usize + self.terminal_row, 0xFFFFFFFF);
        //         } else {
        //             self.draw_pixel(cx + self.terminal_col, cy as usize + self.terminal_row, 0x00);
        //         }
        //         index -= 1;
        //     }
        // }
    }

    pub fn draw_pixel(&mut self, x: u64, y: u64, byte: u32) {
        unsafe {
            let offset = (0x180000 + (y * self.pitch) + ((x * self.bpp) / 8)) as *mut u32;
            *offset = byte;
        }
    }

    pub fn fill_rect(&mut self, x: u64, y: u64, mut width: u64, mut height: u64) {
        // Adjust for overflow
        if (width + x) > SCREEN_WIDTH { width = SCREEN_WIDTH; }
        if (height + x) > SCREEN_HEIGHT { width = SCREEN_HEIGHT; }
        let colour = self.get_rand_colour();

        for i in y..height {
            for j in x..width {
                self.draw_pixel(i, j, colour);
            }
        }
    }

    fn get_rand_colour(&mut self) -> u32 {
        self.colour_num += 1;
        return 0x00000000 + (self.colour_num * 100 * 0xFF) as u32;
    }
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}