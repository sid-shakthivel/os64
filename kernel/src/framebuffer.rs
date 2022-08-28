// src/framebuffer.rs

/*
    Framebuffer is portion of RAM whi&ch contains a bitmap which maps to the display (pixels)
    GRUB sets the correct video mode before loading the &kernel as specified within the multiboot header
    Pitch is number of bytes per row, BPP is bit depth
    Rectangles are arranged like this:
        Top
    Left    Right
        Bottom
    The order of windows is maintained through the stack in which the top most window is at the front and the bottom window is at the back
    Clipping is a method to enable/disable rendering of certain areas by only rendering the topmost pixels in which overlapping regions are not rendered
    A dirty rectangle list is a way to keep track of regions of the screen which need to be repainted which can be used upon the dragging of windows
*/

/*
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16
*/

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::allocator::{kfree, kmalloc};
use crate::spinlock::Lock;
use crate::writer::Writer;
use crate::CONSOLE;
use crate::{list::Stack, print_serial};
use crate::{page_frame_allocator, paging};
use multiboot2::FramebufferTag;
use x86_64::structures::paging::page;

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;
pub const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
pub const BACKGROUND_COLOUR: u32 = 0x3499fe;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x7092be;
const WINDOW_TITLE_HEIGHT: u64 = 20;

pub static DESKTOP: Lock<Window> = Lock::new(Window::new(
    0,
    0,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    None,
    BACKGROUND_COLOUR,
));

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
    colour: u32, // This field may be needed later
    clipped_rectangles: Stack<Rectangle>,
    parent: Option<*mut Window>,
    children: Stack<Window>,
    drag_child: Option<*mut Window>,
    drag_x_offset: u64,
    drag_y_offset: u64,
    mouse_x: u64,
    mouse_y: u64,
}

impl Window {
    pub const fn new(
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        parent: Option<*mut Window>,
        colour: u32,
    ) -> Self {
        Window {
            x,
            y,
            width,
            height,
            colour,
            clipped_rectangles: Stack::<Rectangle>::new(),
            parent,
            children: Stack::<Window>::new(),
            drag_child: None,
            drag_x_offset: 0,
            drag_y_offset: 0,
            mouse_x: 0,
            mouse_y: 0,
        }
    }

    // Creates a new window upon itself (mostly used for the background window)
    pub fn add_sub_window(&mut self, window: Window) {
        self.children.push(window);
    }

    // Paints a window upon the screen
    pub fn paint(&mut self, dirty_rectangles: Stack<Rectangle>, paint_children: bool) {
        // Apply bound clipping to obtain visible parts of window
        self.clip(dirty_rectangles.clone());

        // Actually draw window
        self.draw_window();

        // Clear clipping rects
        self.clipped_rectangles.empty();

        // Optionally paint children (Might be temporary)
        if paint_children {
            for (_i, child) in self.children.into_iter().enumerate() {
                child
                    .unwrap()
                    .payload
                    .clone()
                    .paint(dirty_rectangles.clone(), false);
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse_x: u64, mouse_y: u64, is_left_pressed: bool) {
        // Check if mouse is actually pressed
        if is_left_pressed {
            // Obtain window which mouse is over
            let (index, wrapped_window) = self.get_selected_child(self.mouse_x, self.mouse_y);

            // Raise window
            if let Some(mut window) = wrapped_window {
                window.raise(index);
            }

            // Update location of window being dragged
            if let Some(dragged_window) = self.drag_child {
                unsafe {
                    (*dragged_window).update_location(
                        self.mouse_x,
                        self.mouse_y,
                        self.drag_x_offset,
                        self.drag_y_offset,
                    );
                }
            }
        } else {
            self.drag_child = None;
        }

        let mouse_rect = Rectangle::new(
            self.mouse_x,
            self.mouse_y,
            self.mouse_x + 5,
            self.mouse_y + 5,
        );
        let mut dirty_rectangles = Stack::<Rectangle>::new();
        dirty_rectangles.push(mouse_rect);

        // Repaint given background
        self.paint(dirty_rectangles, true);

        // Paint mouse
        FRAMEBUFFER
            .lock()
            .fill_rect(None, mouse_x, mouse_y, 5, 5, 0x00FF);
        FRAMEBUFFER.free();

        self.mouse_x = mouse_x;
        self.mouse_y = mouse_y;
    }

    /*
        Loops through windows to find window in which mouse coordinates are within
        Returns both the index of the window along with the window itself
    */
    fn get_selected_child(&mut self, mouse_x: u64, mouse_y: u64) -> (usize, Option<Window>) {
        for (i, window) in self.children.into_iter().enumerate() {
            let temp = window.unwrap().payload.clone();
            if mouse_x >= temp.x
                && mouse_x <= (temp.x + temp.width)
                && mouse_y >= temp.y
                && mouse_y <= (temp.y + temp.height)
            {
                // Update drag window, etc
                let const_ptr = &window.unwrap().payload as *const Window;
                let mut_ptr = const_ptr as *mut Window;
                self.drag_child = Some(mut_ptr);
                self.drag_x_offset = mouse_x - temp.x;
                self.drag_y_offset = mouse_y - temp.y;
                return (i, Some(temp));
            }
        }

        return (0, None);
    }

    // Moves window to the top of the stack and trigers a repaint
    fn raise(&mut self, index: usize) {
        if let Some(parent) = self.parent {
            unsafe {
                // Move window if it isn't head (already at the top of the stack)
                if (&*(*parent).children.head.unwrap()).payload.clone() != self.clone() {
                    let address = (*parent).children.remove_at(index);
                    // kfree(address as *mut u64);
                    (*parent).children.push(self.clone());
                }
            }
        }
    }

    // Applies clipping to a window against dirty rectangles, windows, titlebar, children
    fn clip(&mut self, dirty_rectangles: Stack<Rectangle>) {
        let subject_rect = Rectangle::from_window(self);

        // Dirty rectangles are the only regions which need to be updated
        if dirty_rectangles.length > 0 {
            // Add dirty rectangles since these regions must be rerendered
            for (_i, dirty_rectangle) in dirty_rectangles.into_iter().enumerate() {
                self.clipped_rectangles
                    .push(dirty_rectangle.unwrap().payload.clone());
            }
        } else {
            // Clip against self
            self.add_rectangle(&subject_rect);

            for (_i, child) in self.children.clone().into_iter().enumerate() {
                let child_rectangle = Rectangle::from_window(&child.unwrap().payload);
                self.subtract_rectangle(&child_rectangle);
            }

            // Get windows above
            if let Some(parent) = self.parent {
                let mut windows_above =
                    unsafe { (*parent).children.get_higher_nodes(self.clone()) };

                // Apply clipping against windows above
                // Conditional statement exists because windows above includes the background
                for (i, window) in windows_above.into_iter().enumerate() {
                    if i > windows_above.length - 1 {
                        let window_rectangle = Rectangle::from_window(&window.unwrap().payload);
                        self.subtract_rectangle(&window_rectangle);
                    }
                }

                // windows_above.empty();
            }
        }
    }

    // Update coordinates of window whilst taking care to update other windows
    fn update_location(
        &mut self,
        mouse_x: u64,
        mouse_y: u64,
        drag_x_offset: u64,
        drag_y_offset: u64,
    ) {
        self.clip(Stack::<Rectangle>::new());

        // Make a new rect with the updated coordinates in order to clip the subject
        let new_x = mouse_x.wrapping_sub(drag_x_offset);
        let new_y = mouse_y.wrapping_sub(drag_y_offset);

        let clipping_rect = Rectangle::new(new_x, new_y, new_x + self.width, new_y + self.height);

        // Extract out section which doesn't need to be updated (overlap) and returns sections which need to be updated
        self.subtract_rectangle(&clipping_rect);

        // Sections that need to be updated are dirty rectangles
        let dirty_rectangles = self.clipped_rectangles.clone();

        self.clipped_rectangles.empty();

        // Finally update our coordinates which updates position
        self.x = new_x;
        self.y = new_y;

        // All windows below the current window may need to be updated upon a move

        // Ensure that there are regions which need to be updated before updating
        if dirty_rectangles.length > 0 {
            if let Some(parent) = self.parent {
                unsafe {
                    // Repaint parent given the dirty regions (most likely the background)
                    (*parent).paint(dirty_rectangles.clone(), false);

                    // Repaint windows below the moving window
                    let mut windows_below = (*parent).children.get_lower_nodes(self.clone());
                    for (_i, window) in windows_below.into_iter().enumerate() {
                        window
                            .unwrap()
                            .payload
                            .clone()
                            .paint(dirty_rectangles.clone(), false);
                    }
                    // windows_below.empty();
                }
            }
        }

        self.paint(Stack::<Rectangle>::new(), false);
    }

    /*
        Subtracts regions of an rectangle from another
        Loops over all previous clips to ensure everything works
    */
    fn subtract_rectangle(&mut self, clipping_rect: &Rectangle) {
        // Loop through the clipping rects
        let mut index = 0;

        while index < self.clipped_rectangles.length {
            let mut raw = self.clipped_rectangles.get_at(index);

            // If the clipping rect intersects with a rectangle (subject) split it or else move onto the next one
            if clipping_rect.left < raw.right
                && clipping_rect.right > raw.left
                && clipping_rect.top < raw.bottom
                && clipping_rect.bottom > raw.top
            {
                // Remove old rectangle and replace with list of rects
                self.clipped_rectangles.append(raw.split(clipping_rect));
                self.clipped_rectangles.remove_at(index);

                // Reset the counter
                index = 0;
            } else {
                index += 1;
            }
        }
    }

    fn add_rectangle(&mut self, clipping_rect: &Rectangle) {
        self.subtract_rectangle(clipping_rect);
        self.clipped_rectangles.push(*clipping_rect);
    }

    fn draw_window(&mut self) {
        if self.parent.is_some() {
            // If window has parent, implies it is a proper window

            // Paint the main window portion
            FRAMEBUFFER.lock().fill_rect(
                Some(&self.clipped_rectangles),
                self.x + 3,
                self.y + WINDOW_TITLE_HEIGHT + 3,
                self.width - 6,
                self.height - WINDOW_TITLE_HEIGHT - 6,
                self.colour,
            );
            FRAMEBUFFER.free();

            // Paint the top title bar
            FRAMEBUFFER.lock().fill_rect(
                Some(&self.clipped_rectangles),
                self.x + 3,
                self.y + 3,
                self.width - 6,
                WINDOW_TITLE_HEIGHT,
                WINDOW_TITLE_COLOUR,
            );
            FRAMEBUFFER.free();

            // Paint the outline of the window
            FRAMEBUFFER.lock().draw_rect_outline(
                Some(&self.clipped_rectangles),
                self.x,
                self.y,
                self.width,
                self.height,
                WINDOW_BORDER_COLOUR,
            );
            FRAMEBUFFER.free();
        } else {
            // Most likely the background

            FRAMEBUFFER.lock().fill_rect(
                Some(&self.clipped_rectangles),
                self.x,
                self.y,
                self.width,
                self.height,
                self.colour,
            );
            FRAMEBUFFER.free();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    top: u64,
    bottom: u64,
    left: u64,
    right: u64,
}

impl Rectangle {
    pub fn new(left: u64, top: u64, right: u64, bottom: u64) -> Self {
        Rectangle {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
        }
    }

    pub fn from_window(window: &Window) -> Self {
        // print_serial!("{:?} {:p}\n", window, window as *const Window);
        Self::new(
            window.x,
            window.y,
            window.x + window.width,
            window.y + window.height,
        )
    }

    /*
        Method is called upon the subject rect (bottom), given a clipping rect (top)
        Returns a list of rectangles that can be drawn by splitting subject by clipping upon various axes
        WARNING: Might need to edit clipping_rect rather then self
    */
    fn split(&mut self, clipping_rect: &Rectangle) -> Stack<Rectangle> {
        let mut split_rectangles = Stack::<Rectangle>::new();

        // Check if clipping rect left side intersects with subject
        if clipping_rect.left > self.left && clipping_rect.left < self.right {
            // Make new rect with updated coordinates
            let new_rect = Rectangle::new(self.left, self.top, clipping_rect.left, self.bottom);

            // Update current rectangle to match (update left)
            self.left = clipping_rect.left;

            split_rectangles.push(new_rect);
        }

        // Check if clipping rect top side intersects with subject
        if clipping_rect.top > self.top && clipping_rect.top < self.bottom {
            let new_rect = Rectangle::new(self.left, self.top, self.right, clipping_rect.top);

            // Update current rectange to match (update top)
            self.top = clipping_rect.top;

            split_rectangles.push(new_rect);
        }

        // Check if clipping rect right side intersects with subject
        if clipping_rect.right > self.left && clipping_rect.right < self.right {
            let new_rect = Rectangle::new(clipping_rect.right, self.top, self.right, self.bottom);
            self.right = clipping_rect.right;

            split_rectangles.push(new_rect);
        }

        // Check if clipping rect bottom intersects with subject
        if clipping_rect.bottom > self.top && clipping_rect.bottom < self.bottom {
            let new_rect = Rectangle::new(self.left, clipping_rect.bottom, self.right, self.bottom);
            self.bottom = clipping_rect.bottom;

            split_rectangles.push(new_rect);
        }

        return split_rectangles;
    }
}

pub struct Framebuffer {
    pitch: u64,
    bpp: u64,
    backbuffer: u64,
    frontbuffer: u64,
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

pub static FRAMEBUFFER: Lock<Framebuffer> = Lock::new(Framebuffer::new());

impl Framebuffer {
    pub const fn new() -> Framebuffer {
        Framebuffer {
            pitch: 0,
            bpp: 0,
            backbuffer: 0,
            frontbuffer: 0,
        }
    }

    /*  Recives a list of clipped rectangles (these rectangles are to be rendered upon the screen)
        Loops through that list and clamps the clipping rects to the window before drawing
        If clipped rectangles are not supplied, simply clamps to the screen and draws pixels
    */
    pub fn fill_rect(
        &mut self,
        clipped_rectangles: Option<&Stack<Rectangle>>,
        mut x: u64,
        mut y: u64,
        width: u64,
        height: u64,
        colour: u32,
    ) {
        match clipped_rectangles {
            Some(rectangles) => {
                for (_i, clipped_rectangle) in rectangles.into_iter().enumerate() {
                    let clip = clipped_rectangle.unwrap().payload;

                    // Clamp printable area to clipped region itself
                    let x_base = core::cmp::max(x, clip.left);
                    let y_base = core::cmp::max(y, clip.top);
                    let x_limit = core::cmp::min(x + width, clip.right);
                    let y_limit = core::cmp::min(y + height, clip.bottom);

                    for i in x_base..x_limit {
                        for j in y_base..y_limit {
                            self.draw_pixel(i, j, colour);
                        }
                    }
                }
            }
            None => {
                x = core::cmp::max(x, 0);
                y = core::cmp::max(y, 0);

                if x.checked_add(width).is_some() && y.checked_add(height).is_some() {
                    let x_limit = core::cmp::min(x + width, SCREEN_WIDTH);
                    let y_limit = core::cmp::min(y + height, SCREEN_HEIGHT);

                    for i in x..x_limit {
                        for j in y..y_limit {
                            self.draw_pixel(i, j, colour);
                        }
                    }
                }
            }
        }
    }

    // Draws outline of rectangle only
    pub fn draw_rect_outline(
        &mut self,
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        colour: u32,
    ) {
        if x.checked_add(width).is_some() && y.checked_add(height).is_some() {
            // Top bar
            self.draw_horizontal_line(clipped_rectangles, x, y, width, colour);

            // Bottom bar
            self.draw_horizontal_line(clipped_rectangles, x, y + height - 3, width, colour);

            // Left bar
            self.draw_vertical_line(clipped_rectangles, x, y, height, colour);

            // Right bar
            self.draw_vertical_line(clipped_rectangles, x + width - 3, y, height, colour);
        }
    }

    pub fn draw_pixel(&mut self, x: u64, y: u64, byte: u32) {
        let offset = (self.frontbuffer + (y * 4096) + ((x * 32) / 8)) as *mut u32;
        unsafe {
            *offset = byte;
        }
    }

    pub fn write_to_frontbuffer(&mut self) {
        let backbuffer_p = self.backbuffer as *mut u8;
        let frontbuffer_p = self.frontbuffer as *mut u8;

        for i in 0..3145728 {
            unsafe {
                *frontbuffer_p.offset(i) = *backbuffer_p.offset(i);
            }
        }
    }

    fn draw_horizontal_line(
        &mut self,
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        length: u64,
        colour: u32,
    ) {
        self.fill_rect(clipped_rectangles, x, y, length, 3, colour);
    }

    fn draw_vertical_line(
        &mut self,
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        length: u64,
        colour: u32,
    ) {
        self.fill_rect(clipped_rectangles, x, y, 3, length, colour);
    }

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
}

impl Writer for Framebuffer {
    fn clear(&mut self) {
        self.fill_rect(None, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00);
    }

    fn put_char(&mut self, character: char) {
        match character {
            _ => self.draw_character(character),
        }
    }
}

/*
    Utilise 2 buffers which are used to write to graphics memory
    Front buffer - buffer which maps to video memory
    Backbuffer - buffer in which work is completed upon
    Swapping the buffers refers to when memory is copied from backbuffer to frontbuffer
    Main advantage is that users do not see pixel modification, writing to vm is expensive
*/

pub fn init(framebuffer_tag: FramebufferTag) {
    // Setup font information
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let _font = unsafe { &*(font_start as *const PsfFont) };

    FRAMEBUFFER.lock().pitch = framebuffer_tag.pitch as u64;
    FRAMEBUFFER.free();
    FRAMEBUFFER.lock().bpp = framebuffer_tag.bpp as u64;
    FRAMEBUFFER.free();

    // Calulate sizes of the framebuffer
    let size_in_bytes = ((framebuffer_tag.bpp as u64)
        * (framebuffer_tag.width as u64)
        * (framebuffer_tag.height as u64))
        / 8;

    let size_in_mb = page_frame_allocator::convert_bytes_to_mb(size_in_bytes);

    // Setup the front buffer
    let frontbuffer_address = kmalloc(size_in_bytes) as u64;

    // // Identity map this buffer so it maps to video memory
    paging::identity_map_from(framebuffer_tag.address, frontbuffer_address, size_in_mb);

    FRAMEBUFFER.lock().frontbuffer = frontbuffer_address;
    FRAMEBUFFER.free();

    // Setup the back buffer
    // let backbuffer_address = malloc(size_in_bytes) as u64;

    FRAMEBUFFER.lock().backbuffer = 0;
    FRAMEBUFFER.free();
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
