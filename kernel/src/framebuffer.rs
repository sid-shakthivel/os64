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

// TODO: Make a malloc
// TODO: Refactor everything (Make a trait to handle clear, paint, etc)

#![allow(dead_code)]

use crate::list::Stack;
use crate::paging;
use crate::spinlock::Lock;
use crate::writer::Writer;
use lazy_static::lazy_static;
use multiboot2::FramebufferTag;

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;

#[derive(Debug, Clone)]
pub struct Desktop {
    windows: Stack<Window>,
    rectangles: Stack<Rectangle>,
    colour_num: usize,
    drag_window: Option<*mut Window>,
    drag_x_offset: u64,
    drag_y_offset: u64,
}

pub static DESKTOP: Lock<Desktop> = Lock::new(Desktop::new());

impl Desktop {
    pub const fn new() -> Desktop {
        Desktop {
            windows: Stack::<Window>::new(),
            rectangles: Stack::<Rectangle>::new(),
            colour_num: 0,
            drag_window: None,
            drag_x_offset: 0,
            drag_y_offset: 0,
        }
    }

    pub fn create_window(&mut self, x: u64, y: u64, width: u64, height: u64) {
        let new_window = Window::new(x, y, width, height, self.get_rand_colour());

        self.windows.push(
            PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
            new_window,
        );
        PAGE_FRAME_ALLOCATOR.free();

        // Create new rectangle which are rendered upon the screen
        let new_rectangle = Rectangle::new(
            new_window.y,
            new_window.y + new_window.height,
            new_window.x,
            new_window.x + new_window.width,
        );

        // self.add_rectangle(new_rectangle);
    }

    pub fn paint(&self, mouse_x: u64, mouse_y: u64) {
        // Paint windows
        for (_i, window) in self.windows.into_iter().enumerate() {
            window.unwrap().payload.paint();
        }

        for (_i, rectangle) in self.rectangles.into_iter().enumerate() {
            rectangle.unwrap().payload.paint(0xFF);
        }

        // Paint mouse
        Framebuffer::fill_rect(mouse_x, mouse_y, 5, 5, 0xFF);
    }

    // Move z order of window to top and handle dragging of windows (stack represents z order)
    pub fn handle_mouse_movement(&mut self, mouse_x: u64, mouse_y: u64) {
        self.update_drag_window_coordinates(mouse_x, mouse_y);
        let (index, window) = self.get_clicked_window(mouse_x, mouse_y);

        if window.is_some() {
            let unwrapped_window = window.unwrap().clone();

            // Remove from linked list
            self.windows.remove_at(index);

            // TODO: Check if this works effectively (reuse same memory location instead of freeing first)
            // self.windows.push(&mut unwrapped_window as *mut u64, &mut unwrapped_window);
        }
    }

    fn add_rectangle(&mut self, mut new_rectangle: Rectangle) {
        let mut self_clone = self.clone();
        for (_i, rectangle) in self_clone.rectangles.into_iter().enumerate() {
            let mut unwrapped_rect = rectangle.unwrap().payload;
            // Check for overlap within rectangles and accomodate clipped rects
            if !(unwrapped_rect.left <= new_rectangle.right
                && unwrapped_rect.right >= new_rectangle.left
                && unwrapped_rect.top <= new_rectangle.bottom
                && unwrapped_rect.bottom >= new_rectangle.top)
            {
                continue;
            }

            // Split and add to rect's linked list
            let split_rectangles = Rectangle::split(&mut unwrapped_rect, &mut new_rectangle);
            unsafe {
                (*split_rectangles.tail.unwrap()).next = self.rectangles.head;
            }
            self.rectangles.head = split_rectangles.head;
        }

        self.rectangles.push(
            PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
            new_rectangle,
        );
    }

    // Loop through windows and find appropriate one
    fn get_clicked_window(&mut self, mouse_x: u64, mouse_y: u64) -> (usize, Option<Window>) {
        for (i, window) in self.windows.into_iter().enumerate() {
            let mut temp = window.unwrap().payload;
            if mouse_x >= temp.x
                && mouse_x <= (temp.x + temp.width)
                && mouse_y >= temp.y
                && mouse_y <= (temp.y + temp.height)
            {
                // Update drag window, etc
                self.drag_window = Some(&mut temp as *mut Window);
                self.drag_x_offset = mouse_x - temp.x;
                self.drag_y_offset = mouse_y - temp.y;
                return (i, Some(temp));
            }
        }

        return (0, None);
    }

    // Update position of dragged window to mouse coordinates
    fn update_drag_window_coordinates(&mut self, mouse_x: u64, mouse_y: u64) {
        if self.drag_window.is_some() {
            let window = unsafe { &mut *self.drag_window.unwrap() };
            window.clear();
            window.x = mouse_x.wrapping_sub(self.drag_x_offset);
            window.y = mouse_y.wrapping_sub(self.drag_y_offset);
        }
    }

    fn get_rand_colour(&mut self) -> u32 {
        self.colour_num += 1;
        return 0x00000000 + (self.colour_num * 100 * 0xFF00) as u32;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Window {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
    colour: u32,
}

impl Window {
    pub fn new(x: u64, y: u64, width: u64, height: u64, colour: u32) -> Self {
        Window {
            x,
            y,
            width,
            height,
            colour,
        }
    }

    pub fn clear(&self) {
        Framebuffer::fill_rect(self.x, self.y, self.width, self.height, 0x00);
    }

    pub fn paint(&self) {
        Framebuffer::draw_rect(self.x, self.y, self.width, self.height, self.colour);
    }
}

#[derive(Debug, Copy, Clone)]
struct Rectangle {
    top: u64,
    bottom: u64,
    right: u64,
    left: u64,
}

impl Rectangle {
    pub fn new(top: u64, bottom: u64, left: u64, right: u64) -> Self {
        Rectangle {
            top: top,
            bottom: bottom,
            right: right,
            left: left,
        }
    }

    /*
        Method is called upon the subject rect (bottom), given a clipping rect (top)
        Returns a list of rectangles that can be drawn
    */
    fn split(&mut self, clipping_rect: &mut Rectangle) -> Stack<Rectangle> {
        let mut split_rectangles = Stack::<Rectangle>::new();

        // Check if clipping rect left side intersects with subject
        if clipping_rect.left > self.left && clipping_rect.left < self.right {
            // Make new rect with updated coordinates
            let new_rect = Rectangle::new(self.top, self.bottom, self.left, clipping_rect.left);

            // Update current rectangle to match (update left)
            self.left = new_rect.right;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
        }

        // Check if clipping rect top side intersects with subject
        if clipping_rect.top > self.top && clipping_rect.top < self.bottom {
            let new_rect = Rectangle::new(self.top, clipping_rect.top, self.left, self.right);

            // Update current rectange to match (update top)
            self.top = new_rect.bottom;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
        }

        // Check if clipping rect right side intersects with subject
        if clipping_rect.right > self.left && clipping_rect.right < self.right {
            let new_rect = Rectangle::new(self.top, self.bottom, self.left, clipping_rect.left);
            self.left = clipping_rect.left;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
        }

        // Check if clipping rect bottom intersects with subject
        if clipping_rect.bottom > self.top && clipping_rect.bottom < self.bottom {
            let new_rect = Rectangle::new(self.top, clipping_rect.top, self.left, self.right);
            self.left = clipping_rect.left;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
        }

        return split_rectangles;
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
