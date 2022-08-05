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

// TODO: Free memory and make a malloc

#![allow(dead_code)]

use crate::paging;
use crate::spinlock::Lock;
use crate::writer::Writer;
use lazy_static::lazy_static;
use multiboot2::FramebufferTag;

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::print_serial;
use crate::CONSOLE;

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
    head_rect: Option<*mut Rectangle>,
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
            head_rect: None,
        }
    }

    pub fn create_window(&mut self, x: u64, y: u64, width: u64, height: u64) {
        let p_window: *mut Window = PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as *mut _;
        PAGE_FRAME_ALLOCATOR.free();

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

        // Create new rectangle which are rendered upon the screen
        let new_rectangle = Rectangle::new(
            window.y,
            window.y + window.height,
            window.x,
            window.x + window.width,
        );
        self.add_clipped_rect(new_rectangle);
    }

    fn add_clipped_rect(&mut self, new_rectangle: &mut Rectangle) {
        let mut current_rect = self.head_rect;
        if current_rect.is_some() {
            let unwrapped_rect = unsafe { &mut *(current_rect.unwrap()) };

            // Check for overlap with other rectangles
            // If there is no overlap, no need for snipping so skip
            if !(unwrapped_rect.left <= new_rectangle.right
                && unwrapped_rect.right >= new_rectangle.left
                && unwrapped_rect.top <= new_rectangle.bottom
                && unwrapped_rect.bottom >= new_rectangle.top)
            {
                // continue;
            } else {
                // Split and add to rect's linked list
                let rectangles = Rectangle::split(unwrapped_rect, new_rectangle);

                // Remove subject rect and replace with the split ones TODO: Temp fix
                self.head_rect = rectangles;

                current_rect = unwrapped_rect.next;
            }
        }

        // Add to rectangle list
        if self.head_rect.is_some() {
            new_rectangle.next = self.head_rect;
        }
        self.head_rect = Some(new_rectangle as *mut Rectangle);
    }

    // Current functionality is to delete a window if pressed
    pub fn handle_mouse_movement(&mut self, mouse_x: u64, mouse_y: u64) {
        // self.update_drag_window_coordinates(mouse_x, mouse_y);
        let window = self.get_clicked_window(mouse_x, mouse_y);

        if window.is_some() {
            // let unwrapped_window = window.unwrap().clone();

            // Remove from linked list
            // self.remove_window(&unwrapped_window);

            // Add to top of linked list so it appears at the start
        }

        // Paint
        self.paint(mouse_x, mouse_y);
    }

    // Update coordinates of a dragged window in relation to mouse cursor
    fn update_drag_window_coordinates(&mut self, mouse_x: u64, mouse_y: u64) {
        if self.drag_window.is_some() {
            unsafe {
                let window = &mut *self.drag_window.unwrap();
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

        let mut current_rect = self.head_rect;
        while current_rect.is_some() {
            let unwrapped_rect = unsafe { &mut (*current_rect.unwrap()) };
            print_serial!("{:?}\n", unwrapped_rect);
            unwrapped_rect.paint(0xFF);
            current_rect = unsafe { (*current_rect.unwrap()).next };
        }

        print_serial!("Painting\n");

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

        Framebuffer::draw_rect(
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

#[derive(Debug)]
struct Rectangle {
    top: u64,
    bottom: u64,
    right: u64,
    left: u64,
    next: Option<*mut Rectangle>,
}

impl Rectangle {
    pub fn new(top: u64, bottom: u64, left: u64, right: u64) -> &'static mut Rectangle {
        let p_rectangle: *mut Rectangle =
            PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as *mut _;
        PAGE_FRAME_ALLOCATOR.free();
        let rectangle = unsafe { &mut *p_rectangle };
        rectangle.top = top;
        rectangle.bottom = bottom;
        rectangle.left = left;
        rectangle.right = right;
        rectangle
    }

    /*
        Method is called upon the subject rect, given a clipping rect
        Returns a list of rectangles that can be drawn
    */
    fn split(&mut self, clipping_rect: &mut Rectangle) -> Option<*mut Rectangle> {
        let mut new_rectangles: Option<*mut Rectangle> = None;
        print_serial!("Splitting rect\n");
        // Check left side of subject to right side of clipping
        if clipping_rect.left > self.left && clipping_rect.left < self.right {
            // Make new rect with updated coordinates
            let new_rect = Rectangle::new(self.top, self.bottom, self.left, clipping_rect.left);

            // Update current rectangle to match (update left)
            self.left = new_rect.right;

            if new_rectangles.is_none() {
                new_rectangles = Some(new_rect as *mut Rectangle);
            } else {
                unsafe {
                    (*new_rectangles.unwrap()).next = Some(new_rect as *mut Rectangle);
                }
            }
        }

        // Compare subject bottom to clipping top
        if clipping_rect.top > self.top && clipping_rect.top < self.bottom {
            let new_rect = Rectangle::new(self.top, clipping_rect.top, self.left, self.right);

            // Update current rectange to match (update top)
            self.top = new_rect.bottom;

            if new_rectangles.is_none() {
                new_rectangles = Some(new_rect as *mut Rectangle);
            } else {
                unsafe {
                    (*new_rectangles.unwrap()).next = Some(new_rect as *mut Rectangle);
                }
            }
        }

        if clipping_rect.right > self.left && clipping_rect.right < self.right {
            let new_rect = Rectangle::new(self.top, self.bottom, self.left, clipping_rect.left);
            self.left = clipping_rect.left;

            if new_rectangles.is_none() {
                new_rectangles = Some(new_rect as *mut Rectangle);
            } else {
                unsafe {
                    (*new_rectangles.unwrap()).next = Some(new_rect as *mut Rectangle);
                }
            }
        }

        if clipping_rect.bottom > self.top && clipping_rect.bottom < self.bottom {
            let new_rect = Rectangle::new(self.top, clipping_rect.top, self.left, self.right);
            self.left = clipping_rect.left;

            if new_rectangles.is_none() {
                new_rectangles = Some(new_rect as *mut Rectangle);
            } else {
                unsafe {
                    (*new_rectangles.unwrap()).next = Some(new_rect as *mut Rectangle);
                }
            }
        }

        return new_rectangles;
    }

    fn paint(&self, colour: u32) {
        Framebuffer::draw_rect(
            self.left,
            self.top,
            self.right - self.left,
            self.bottom - self.top,
            colour,
        );
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

    // Draws rectangle
    pub fn fill_rect(x: u64, y: u64, width: u64, height: u64, colour: u32) {
        if x.checked_add(width).is_some() && y.checked_add(height).is_some() {
            for i in x..(width + x) {
                for j in y..(height + y) {
                    Self::draw_pixel(i, j, colour);
                }
            }
        }
    }

    // Draws outline of rectangle only
    pub fn draw_rect(x: u64, y: u64, width: u64, height: u64, colour: u32) {
        Framebuffer::draw_horizontal_line(x, y, width, colour);
        Framebuffer::draw_horizontal_line(x, y + height, width, colour);

        Framebuffer::draw_vertical_line(x, y, height, colour);
        Framebuffer::draw_vertical_line(x + width, y, height, colour);
    }

    pub fn draw_pixel(x: u64, y: u64, byte: u32) {
        unsafe {
            // TODO: make this use framebuffer array for safety
            let offset = (0x180000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
            *offset = byte;
        }
    }

    fn draw_horizontal_line(x: u64, y: u64, length: u64, colour: u32) {
        Framebuffer::fill_rect(x, y, length, 5, colour);
    }

    fn draw_vertical_line(x: u64, y: u64, length: u64, colour: u32) {
        Framebuffer::fill_rect(x, y, 5, length, colour);
    }
}

impl Writer for Framebuffer {
    fn clear(&mut self) {
        Framebuffer::fill_rect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00);
    }

    fn put_char(&mut self, character: char) {
        match character {
            _ => self.draw_character(character),
        }
    }
}

pub fn init(framebuffer_tag: FramebufferTag) {
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let _font = unsafe { &*(font_start as *const PsfFont) };

    FRAMEBUFFER.lock().pitch = framebuffer_tag.pitch as u64;
    FRAMEBUFFER.lock().bpp = framebuffer_tag.bpp as u64;

    paging::identity_map_from(framebuffer_tag.address, 3);
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
