// src/framebuffer.rs

/*
    Framebuffer is portion of RAM which contains a bitmap which maps to the display (pixels)
    GRUB sets the correct video mode before loading the &kernel as specified within the multiboot header
    Pitch is& n&umber of bytes per row, BPP is bit depth
    Rectangles are arranged like this:
        Top
    Left    Right
        Bottom
    Clipping is a method to enable/disable rendering of certain areas by only rendering the topmost pixels in which overlapping regions are not rendered
    A dirty rectangle list is a way to keep track of regions of the screen which need to be repainted which can be used upon the dragging of windows
*/

/*
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16
*/

// TODO: Make a trait to handle clear, paint, etc

use crate::interrupts::new_process_rsp;
use crate::spinlock::Lock;
use crate::writer::Writer;
use crate::CONSOLE;
use crate::{list::Stack, print_serial};
use crate::{page_frame_allocator, paging};
use lazy_static::lazy_static;
use multiboot2::FramebufferTag;

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;
const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
pub const BACKGROUND_COLOUR: u32 = 0x3499fe;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x7092be;
const WINDOW_TITLE_HEIGHT: u64 = 20;

#[derive(Debug, Clone)]
pub struct Desktop {
    pub window: Window,
}

// pub static DESKTOP: Lock<Desktop> = Lock::new(Desktop::new());

pub static DESKTOP: Lock<Window> = Lock::new(Window::new(
    0,
    0,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    None,
    BACKGROUND_COLOUR,
));

impl Desktop {
    // The order of windows is maintained through the stack in which the top most window is at the front and the bottom window is at the back
    // pub const fn new() -> Desktop {
    //     Desktop {
    //         window: Window::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, None, BACKGROUND_COLOUR)
    //     }
    // }

    // Move window to front and handle dragging of windows
    // pub fn handle_mouse_movement(&mut self, mouse_x: u64, mouse_y: u64, is_left_pressed: bool) {
    //     if is_left_pressed {
    //         // self.move_window(mouse_x, mouse_y);
    //         let (index, window) = self.get_clicked_window(mouse_x, mouse_y);
    //         if window.is_some() {
    //             // self.raise_window(window, index);
    //         }
    //     } else {
    //         self.drag_window = None;
    //     }

    //     FRAMEBUFFER
    //         .lock()
    //         .fill_rect(None, mouse_x, mouse_y, 5, 5, 0x00);
    //     FRAMEBUFFER.free();
    // }

    /*
        Loops through windows to find window in which mouse coordinates are within
        Returns both the index of the window along with the window itself
    */
    // fn get_clicked_window(&mut self, mouse_x: u64, mouse_y: u64) -> (usize, Option<Window>) {
    //     for (i, window) in self.windows.into_iter().enumerate() {
    //         let temp = window.unwrap().payload.clone();
    //         if mouse_x >= temp.x
    //             && mouse_x <= (temp.x + temp.width)
    //             && mouse_y >= temp.y
    //             && mouse_y <= (temp.y + temp.height)
    //         {
    //             // Update drag window, etc
    //             let const_ptr = &window.unwrap().payload as *const Window;
    //             let mut_ptr = const_ptr as *mut Window;
    //             self.drag_window = Some(mut_ptr);
    //             self.drag_x_offset = mouse_x - temp.x;
    //             self.drag_y_offset = mouse_y - temp.y;
    //             return (i, Some(temp));
    //         }
    //     }

    //     return (0, None);
    // }

    // Moves window to the top of the stack
    // fn raise_window(&mut self, window: Option<Window>, index: usize) {
    //     // WARNING: Only do this maneuver if it's not head for performance
    //     let unwrapped_window = window.unwrap().clone();

    //     // Remove from linked list
    //     self.windows.remove_at(index);

    //     // WARNING: Should really preserve old window and move it (make a method within list)
    //     self.windows.push(
    //         PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64,
    //         unwrapped_window,
    //     );
    //     PAGE_FRAME_ALLOCATOR.free();
    // }
}

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
        }
    }

    // Creates a new window upon itself (mostly used for the background window)
    pub fn add_sub_window(&mut self, window: Window) {
        self.children
            .push(PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64, window);
        PAGE_FRAME_ALLOCATOR.free();
    }

    // Paints a window upon the screen
    pub fn paint(&mut self, dirty_rectangles: Stack<Rectangle>, paint_children: bool) {
        // Apply bound clipping to obtain visible parts of window
        self.clip(dirty_rectangles.clone());

        // Apply titlebar logic and extract from window (for now ain't doing)

        // Actually draw window
        self.draw_window();

        // Clear clipping rects
        // self.clipped_rectangles.empty();

        // Optionally paint children (Might be temporary)
        if paint_children {
            for (i, child) in self.children.into_iter().enumerate() {
                // child
                //     .unwrap()
                //     .payload
                //     .clone()
                //     .paint(dirty_rectangles.clone(), false);
            }
        }
    }

    // Applies clipping to a window against dirty rectangles, windows, titlebar, children
    fn clip(&mut self, dirty_rectangles: Stack<Rectangle>) {
        let mut subject_rect = Rectangle::from_window(self);

        // Apply clipping against titlebar
        let titlebar_rectangle = Rectangle::new(
            self.x,
            self.y,
            self.x + self.width,
            self.y + self.y + WINDOW_TITLE_HEIGHT,
        );

        // self.clipped_rectangles = subject_rect.subtract_rectangle(&titlebar_rectangle);

        // Apply clipping against dirty rectangles
        for (i, dirty_rectangle) in dirty_rectangles.into_iter().enumerate() {
            self.clipped_rectangles
                .append(subject_rect.subtract_rectangle(&dirty_rectangle.unwrap().payload.clone()));
        }

        // Apply clipping against children
        for (i, child) in self.children.into_iter().enumerate() {
            let child_rectangle = Rectangle::from_window(&child.unwrap().payload);

            print_serial!("{:?}\n", child_rectangle);

            // self.clipped_rectangles
            //     .append(subject_rect.subtract_rectangle(&child_rectangle));

            let test = subject_rect.subtract_rectangle(&child_rectangle);
            self.clipped_rectangles.head = test.head;
        }

        // Get windows above
        if let Some(parent) = self.parent {
            let windows_above = unsafe { (*parent).children.get_higher_nodes(self.clone()) };

            // Apply clipping against windows above
            for (i, window) in windows_above.into_iter().enumerate() {
                let window_rectangle = Rectangle::from_window(&window.unwrap().payload);
                self.clipped_rectangles
                    .append(subject_rect.subtract_rectangle(&window_rectangle));
            }
        }
    }

    // Update coordinates of window whilst taking care to update other windows
    fn update_location(&mut self, mouse_x: u64, mouse_y: u64) {
        // Apply clipping to obtain the visible portins of rectangle

        // Update window positions temporally whilst

        // Extract out section which doesn't need to be updated

        // All windows below the current window may need to be updated upon a move

        // Paint parent (most likely the background)

        // Finally update our coordinates and update position

        // Update window coordinates while taking care to preserve the old values

        // let window = unsafe { &mut *self.drag_window.unwrap() };
        // let old_window = window.clone();

        // window.x = mouse_x.wrapping_sub(self.drag_x_offset);
        // window.y = mouse_y.wrapping_sub(self.drag_y_offset);

        // let another_subject_rect = Rectangle::from_window(window);
    }

    fn draw_window(&mut self) {
        // Paint window background
        FRAMEBUFFER.lock().fill_rect(
            Some(&self.clipped_rectangles),
            self.x,
            self.y,
            self.width,
            self.height,
            self.colour,
        );
        FRAMEBUFFER.free();

        // Paint window bar
        // FRAMEBUFFER.lock().fill_rect(
        //     Some(&self.clipped_rectangles),
        //     self.x + 3,
        //     self.y + 3,
        //     self.width - 3,
        //     WINDOW_TITLE_HEIGHT,
        //     WINDOW_TITLE_COLOUR,
        // );
        // FRAMEBUFFER.free();
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
        Self::new(window.x, window.y, window.x + window.width, window.height)
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

            split_rectangles.push(PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64, new_rect);
            PAGE_FRAME_ALLOCATOR.free();
        }

        // Check if clipping rect top side intersects with subject
        if clipping_rect.top > self.top && clipping_rect.top < self.bottom {
            let new_rect = Rectangle::new(self.left, self.top, self.right, clipping_rect.top);

            // Update current rectange to match (update top)
            self.top = clipping_rect.top;

            split_rectangles.push(PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64, new_rect);
            PAGE_FRAME_ALLOCATOR.free();
        }

        // Check if clipping rect right side intersects with subject
        if clipping_rect.right > self.left && clipping_rect.right < self.right {
            let new_rect = Rectangle::new(clipping_rect.right, self.top, self.right, self.bottom);
            self.right = clipping_rect.right;

            split_rectangles.push(PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64, new_rect);
            PAGE_FRAME_ALLOCATOR.free();
        }

        // Check if clipping rect bottom intersects with subject
        if clipping_rect.bottom > self.top && clipping_rect.bottom < self.bottom {
            let new_rect = Rectangle::new(self.left, clipping_rect.bottom, self.right, self.bottom);
            self.bottom = clipping_rect.bottom;

            split_rectangles.push(PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64, new_rect);
            PAGE_FRAME_ALLOCATOR.free();
        }

        return split_rectangles;
    }

    /*
        Punches out regions of rectangle which do not overlap given a clipping rectangle
        Returns a list of rectangles which can be used
    */
    fn subtract_rectangle(&mut self, clipping_rect: &Rectangle) -> Stack<Rectangle> {
        if !(clipping_rect.left <= self.right
            && clipping_rect.right >= self.left
            && clipping_rect.top <= self.bottom
            && clipping_rect.bottom >= self.top)
        {
            panic!("complete overlap?\n");
        }

        self.split(clipping_rect)
    }

    // Applies clipping upon rectangle and then appends the clipping rect upon the list of rects
    fn add_rectangle(&mut self, clipping_rect: &mut Rectangle) -> Stack<Rectangle> {
        let mut split_rectangles = self.subtract_rectangle(clipping_rect);
        split_rectangles.push(
            PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as u64,
            *clipping_rect,
        );
        PAGE_FRAME_ALLOCATOR.free();

        split_rectangles
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

    /*  Recives a list of clipped rectangles (these rectangles are not to be rendered upon the screen)
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

                    // Clamp of printable area to clipped region itself
                    let x_base = core::cmp::max(x, clip.left);
                    let y_base = core::cmp::max(y, clip.top);
                    let x_limit = core::cmp::min(x + width, clip.right);
                    let y_limit = core::cmp::min(y + height, clip.bottom);

                    print_serial!("{} {} {} {}\n", x_base, x_limit, y_base, y_limit);

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
            self.draw_horizontal_line(clipped_rectangles, x, y, width, colour);
            self.draw_horizontal_line(clipped_rectangles, x, y + height, width, colour);

            self.draw_vertical_line(clipped_rectangles, x, y, height, colour);
            self.draw_vertical_line(clipped_rectangles, x + width, y, height, colour);
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
    let size = (framebuffer_tag.bpp as u64)
        * (framebuffer_tag.width as u64)
        * (framebuffer_tag.height as u64);
    let size_mb = page_frame_allocator::convert_bytes_to_mb(size);
    let number_of_pages = page_frame_allocator::convert_bits_to_pages(size);

    // Setup the front buffer
    let frontbuffer_address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(number_of_pages) as u64;
    PAGE_FRAME_ALLOCATOR.free();

    // Identity map this buffer so it maps to video memory
    paging::identity_map_from(framebuffer_tag.address, frontbuffer_address, size_mb);

    FRAMEBUFFER.lock().frontbuffer = frontbuffer_address;
    FRAMEBUFFER.free();

    // Setup the back buffer
    let backbuffer_address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(number_of_pages) as u64;
    PAGE_FRAME_ALLOCATOR.free();

    FRAMEBUFFER.lock().backbuffer = backbuffer_address;
    FRAMEBUFFER.free();
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
