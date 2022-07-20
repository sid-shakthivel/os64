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
    pitch: u32,
    bpp: u32,
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
    pub static ref TEST_TERMINAL: spin::Mutex<Framebuffer> = spin::Mutex::new(Framebuffer {
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

    TEST_TERMINAL.lock().pitch = framebuffer_tag.pitch;
    TEST_TERMINAL.lock().bpp = framebuffer_tag.bpp as u32;
    TEST_TERMINAL.lock().font = Some(*font);
    TEST_TERMINAL.lock().font_start = font_start;

    paging::identity_map_from(framebuffer_tag.address, 3, page_frame_allocator);
}

impl Framebuffer {
    pub fn put_char(&mut self, character: char) {
        // Each glyph is stored in a bitmap of 8 * 16

        let font = self.font.unwrap();
        let test = self.font_start + font.header_size + (font.bytes_per_glyph * (character as u32));

        let glyph_address = test as *mut u8;

        for cx in 0..16 {
            for cy in 0..8 {
                unsafe {
                    let glyph_offset: u16 = (*glyph_address.offset(cx) as u16) & (1 << cy);
                    if glyph_offset > 0 {
                        self.draw_pixel((cy as u32), (cx as u32), 0xFF00);
                    } else {
                        self.draw_pixel((cy as u32), (cx as u32), 0x00FF);
                    }
                }
            }
        }
    }

    fn draw_pixel(&mut self, x: u32, y: u32, byte: u32) {
        let test = y * self.pitch;
        let best = (x * self.bpp) / 8;

        unsafe {
            let offset = (0x180000 + test + best) as *mut u32;
            *offset = byte;
        }
    }
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}