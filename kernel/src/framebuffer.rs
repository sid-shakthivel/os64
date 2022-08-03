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

#![allow(dead_code)]

use crate::spinlock::Lock;
use crate::writer::Writer;
use lazy_static::lazy_static;
use multiboot2::FramebufferTag;

use crate::page_frame_allocator::{Frame, FrameAllocator, PageFrameAllocator};
use crate::print_serial;
use crate::CONSOLE;
use crate::{mouse, paging};

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;

#[derive(Debug, Clone, Copy)]
pub struct Desktop {
    head: Option<*mut Window>,
    tail: Option<*mut Window>,
    count: u64,
    colour_num: usize,
    drag_window: Option<*mut Window>,
    drag_x_offset: u64,
    drag_y_offset: u64,
}

pub static DESKTOP: Lock<Desktop> = Lock::new(Desktop::new());

impl Desktop {
    pub const fn new() -> Desktop {
        Desktop {
            head: None,
            tail: None,
            count: 0,
            colour_num: 0,
            drag_window: None,
            drag_x_offset: 0,
            drag_y_offset: 0,
        }
    }

    pub fn create_window(
        &mut self,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        pf_allocator: &mut PageFrameAllocator,
    ) {
        let p_window: *mut Window = pf_allocator.alloc_frame().unwrap() as *mut _;

        let window = unsafe { &mut *p_window };
        window.init(x, y, width, height, self.get_rand_colour());

        // Link up with linked list
        window.next = self.head;
        if self.head.is_some() {
            unsafe {
                (*self.head.unwrap()).prev = Some(p_window);
            }
        }

        // If first element, make it the tail too
        if self.tail.is_none() {
            self.tail = Some(p_window);
        }

        // Push new window to start of linked list
        self.head = Some(p_window);
        self.count += 1;
    }

    // Current functionality is to delete a window if pressed
    pub fn handle_mouse_movement(&mut self, mouse_x: u64, mouse_y: u64) {
        self.update_drag_window_coordinates(mouse_x, mouse_y);
        let window = self.get_clicked_window(mouse_x, mouse_y);

        if window.is_some() {
            let unwrapped_window = window.unwrap().clone();

            // Remove from linked list
            // self.remove_window(&unwrapped_window);

            // Add to top of linked list
            // unsafe {
            // self.create_window((*unwrapped_window).x, (*unwrapped_window).y, (*unwrapped_window).width, (*unwrapped_window).height, pf_allocator);
            // }
        }

        // Paint
        self.paint(mouse_x, mouse_y);
    }

    // Update coordinates of a dragged window in relation to mouse cursor
    fn update_drag_window_coordinates(&mut self, mouse_x: u64, mouse_y: u64) {
        if !self.drag_window.is_none() {
            unsafe {
                let window = (&mut *self.drag_window.unwrap());
                Framebuffer::fill_rect(window.x, window.y, window.width, window.height, 0x00);
                window.x = mouse_x.wrapping_sub(self.drag_x_offset);
                window.y = mouse_y.wrapping_sub(self.drag_y_offset);
            }
        }
    }

    fn get_clicked_window(&mut self, mouse_x: u64, mouse_y: u64) -> Option<&Window> {
        let mut window = self.head;
        while window.is_some() {
            let unwrapped_window = unsafe { &mut *(window.unwrap()) };
            if mouse_x >= unwrapped_window.x
                && mouse_x <= (unwrapped_window.x + unwrapped_window.width)
                && mouse_y >= unwrapped_window.y
                && mouse_y <= (unwrapped_window.y + unwrapped_window.height)
            {
                // Update drag window, etc
                self.drag_window = Some(unwrapped_window as *mut Window);
                self.drag_x_offset = mouse_x - unwrapped_window.x;
                self.drag_y_offset = mouse_y - unwrapped_window.y;
                return Some(unwrapped_window);
            }
            window = unwrapped_window.next;
        }

        return None;
    }

    fn remove_window(&mut self, target_window: &Window) {
        let mut window = self.head;
        while window.is_some() {
            let unwrapped_window = unsafe { &*(window.unwrap()) };

            if unwrapped_window == target_window {
                // Empty framebuffer
                Framebuffer::fill_rect(
                    unwrapped_window.x,
                    unwrapped_window.y,
                    unwrapped_window.width,
                    unwrapped_window.height,
                    0x00,
                );

                // Configure linked list
                if unwrapped_window.prev.is_some() {
                    unsafe {
                        (*unwrapped_window.prev.unwrap()).next = unwrapped_window.next;
                    }
                } else {
                    // Should replace head
                    unsafe {
                        self.head = (*self.head.unwrap()).next;
                    }
                }

                if unwrapped_window.next.is_some() {
                    unsafe {
                        (*unwrapped_window.next.unwrap()).prev = unwrapped_window.prev;
                    }
                }

                // TODO: Should free memory but need an idiocramatic method to do so or else lose 1 page of memory each time....
                return;
            }

            window = unwrapped_window.next;
        }
        return;
    }

    fn get_rand_colour(&mut self) -> u32 {
        self.colour_num += 1;
        return 0x00000000 + (self.colour_num * 100 * 0xFF00) as u32;
    }

    pub fn paint(&self, mouse_x: u64, mouse_y: u64) {
        // Paint windows
        if self.head.is_some() {
            unsafe { Window::paint(Some(&mut *(self.head.unwrap()))) };
        }

        // Paint mouse
        Framebuffer::fill_rect(mouse_x, mouse_y, 5, 5, 0xFF);
    }
}
// Add chdilren so paint would involve painting children
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Window {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
    next: Option<*mut Window>,
    prev: Option<*mut Window>,
    colour: u32,
}

impl Window {
    pub fn init(&mut self, x: u64, y: u64, width: u64, height: u64, colour: u32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.colour = colour;
    }

    pub fn paint(window: Option<&mut Window>) {
        let window_unwrapped = window.unwrap();

        Framebuffer::fill_rect(
            window_unwrapped.x,
            window_unwrapped.y,
            window_unwrapped.width,
            window_unwrapped.height,
            window_unwrapped.colour,
        );

        if window_unwrapped.next.is_some() {
            let next_window = window_unwrapped.next.unwrap();
            unsafe { return Self::paint(Some(&mut *next_window)) };
        }
    }
}

pub struct Framebuffer {
    framebuffer: &'static mut [u32; 786432],
    pitch: u64,
    bpp: u64,
}

#[derive(Copy, Clone)]
struct PsfFont {
    magic: u32,       // TODO: Work out magic and make a verify func
    version: u32,     // Usually 0
    header_size: u32, // Offset of bitmaps
    flags: u32,
    glymph_num: u32,
    bytes_per_glyph: u32, // Size
    height: u32,          // In pixels
    width: u32,           // In pixels
}

lazy_static! {
    pub static ref FRAMEBUFFER: spin::Mutex<Framebuffer> = spin::Mutex::new(Framebuffer {
        framebuffer: unsafe { &mut *(0x180000 as *mut [u32; 786432]) },
        pitch: 0,
        bpp: 0,
    });
}

impl Framebuffer {
    fn draw_character(&mut self, _character: char) {
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

    pub fn fill_window(&mut self, x: u64, y: u64, mut width: u64, mut height: u64, colour: u32) {
        // Adjust for overflow
        width += x;
        height += y;

        if (width) > SCREEN_WIDTH {
            width = SCREEN_WIDTH;
        }
        if (height) > SCREEN_HEIGHT {
            height = SCREEN_HEIGHT;
        }

        Self::fill_rect(x, y, width, height, colour);
    }

    pub fn fill_rect(x: u64, y: u64, width: u64, height: u64, colour: u32) {
        if x.checked_add(width).is_some() && y.checked_add(height).is_some() {
            for i in x..(width + x) {
                for j in y..(height + y) {
                    Self::draw_pixel(i, j, colour);
                }
            }
        }
    }

    pub fn draw_pixel(x: u64, y: u64, byte: u32) {
        unsafe {
            // TODO: make this use framebuffer array for safety
            let offset = (0x180000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
            *offset = byte;
        }
    }
}

impl Writer for Framebuffer {
    fn clear(&mut self) {
        // TODO: Clear screen
    }

    fn put_char(&mut self, character: char) {
        match character {
            _ => self.draw_character(character),
        }
    }
}

pub fn init(framebuffer_tag: FramebufferTag, page_frame_allocator: &mut PageFrameAllocator) {
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let _font = unsafe { &*(font_start as *const PsfFont) };

    FRAMEBUFFER.lock().pitch = framebuffer_tag.pitch as u64;
    FRAMEBUFFER.lock().bpp = framebuffer_tag.bpp as u64;

    paging::identity_map_from(framebuffer_tag.address, 3, page_frame_allocator);
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
