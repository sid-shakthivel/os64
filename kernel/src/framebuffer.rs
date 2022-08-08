// src/framebuffer.rs

/*
    Framebuffer is portion of RAM which contains a bitmap which maps to the display (pixels)
    GRUB s&ets the correct video mode before loading the &kernel as specified within the multiboot header
    Pitch is n&umber of bytes per row, BPP is bit depth
    Rectangles are arranged like this:
        Top
    Left    Right
        Bottom
    Clipping is a method to enable/disable rendering of certain areas
*/

/*
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16
*/

// TODO: Make a malloc
// TODO: Make a trait to handle clear, paint, etc

use crate::paging;
use crate::spinlock::Lock;
use crate::writer::Writer;
use crate::CONSOLE;
use crate::{list::Stack, print_serial};
use lazy_static::lazy_static;
use multiboot2::FramebufferTag;

use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;
const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
pub const BACKGROUND_COLOUR: u32 = 0x3499fe;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x7092be;

#[derive(Debug, Clone)]
pub struct Desktop {
    windows: Stack<Window>,
    colour_num: usize,
    drag_window: Option<*mut Window>,
    drag_x_offset: u64,
    drag_y_offset: u64,
}

pub static DESKTOP: Lock<Desktop> = Lock::new(Desktop::new());

impl Desktop {
    // The order of windows is maintained through the stack in which the top most window is at the front and the bottom window is at the back
    pub const fn new() -> Desktop {
        Desktop {
            windows: Stack::<Window>::new(),
            colour_num: 0,
            drag_window: None,
            drag_x_offset: 0,
            drag_y_offset: 0,
        }
    }

    pub fn create_window(&mut self, x: u64, y: u64, width: u64, height: u64) {
        let new_window = Window::new(x, y, width, height, WINDOW_BACKGROUND_COLOUR);

        self.windows.push(
            PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
            new_window,
        );
        PAGE_FRAME_ALLOCATOR.free();
    }

    pub fn paint(&mut self, mouse_x: u64, mouse_y: u64) {
        // Paint windows
        for (i, window) in &mut self.windows.into_iter().enumerate() {
            let windows_above = self.windows.get_above_nodes(i);
            window.unwrap().payload.clone().paint(windows_above);
            // WARNING: Should free more memory
        }

        // Paint mouse
        Framebuffer::fill_rect(None, mouse_x, mouse_y, 5, 5, 0x00);
    }

    // Move window to front and handle dragging of windows
    pub fn handle_mouse_movement(&mut self, mouse_x: u64, mouse_y: u64, is_left_pressed: bool) {
        if is_left_pressed {
            self.update_drag_window_coordinates(mouse_x, mouse_y);
            let (index, window) = self.get_clicked_window(mouse_x, mouse_y);

            if window.is_some() {
                let unwrapped_window = window.unwrap().clone();

                // WARNING: Only do this maneuver if it's not head for performance

                // Remove from linked list
                // self.windows.remove_at(index);

                // // WARNING: Should really preserve old window and move it (make a method within list)
                // self.windows.push(
                //     PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                //     unwrapped_window,
                // );
                // PAGE_FRAME_ALLOCATOR.free();
            }
        } else {
            self.drag_window = None;
        }

        self.paint(mouse_x, mouse_y);
    }

    /*
        Loops through windows to find window in which mouse coordinates are within
        Returns both the index of the window along with the window itself
    */
    fn get_clicked_window(&mut self, mouse_x: u64, mouse_y: u64) -> (usize, Option<Window>) {
        for (i, window) in self.windows.into_iter().enumerate() {
            let temp = window.unwrap().payload.clone();
            if mouse_x >= temp.x
                && mouse_x <= (temp.x + temp.width)
                && mouse_y >= temp.y
                && mouse_y <= (temp.y + temp.height)
            {
                // Update drag window, etc
                let const_ptr = &window.unwrap().payload as *const Window;
                let mut_ptr = const_ptr as *mut Window;
                self.drag_window = Some(mut_ptr);
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
            let windows_above = self.windows.get_above_nodes(0);
            window.draw_window(windows_above, false);
            window.x = mouse_x.wrapping_sub(self.drag_x_offset);
            window.y = mouse_y.wrapping_sub(self.drag_y_offset);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
    colour: u32,
    clipped_rectangles: Stack<Rectangle>,
}

impl Window {
    pub fn new(x: u64, y: u64, width: u64, height: u64, colour: u32) -> Self {
        Window {
            x,
            y,
            width,
            height,
            colour,
            clipped_rectangles: Stack::<Rectangle>::new(),
        }
    }

    /*
        Recives a list of all windows above it
        Punches out areas which are covered and sets that to rectangles
        Only renders pixels which are visible thus saving memory
    */
    pub fn paint(&mut self, windows_above: Stack<Window>) {
        self.draw_window(windows_above, true);
    }

    pub fn clear(&mut self, windows_above: Stack<Window>) {
        self.draw_window(windows_above, false);
    }

    fn draw_window(&mut self, windows_above: Stack<Window>, is_colour: bool) {
        let mut self_clone = self.clone();

        // Empty rectangles in case anything has been updated  (should make this a method)
        for (i, rectangle) in &mut self.clipped_rectangles.into_iter().enumerate() {
            self_clone.clipped_rectangles.remove_at(i);
            // TODO: Free properly
        }

        for (i, window) in windows_above.into_iter().enumerate() {
            // Clip subject by windows above to reduce area which is rendered
            let clipping_rect = Rectangle::from_window(&window.unwrap().payload);
            self.subtract_rectangle(clipping_rect); // Apply the clipping rect upon the subject rect
        }

        let mut window_background_colour = self.colour;
        if !is_colour {
            window_background_colour = BACKGROUND_COLOUR;
        }

        let mut window_border_colour = WINDOW_BORDER_COLOUR;
        if !is_colour {
            window_border_colour = BACKGROUND_COLOUR;
        }

        let mut window_title_colour = WINDOW_TITLE_COLOUR;
        if !is_colour {
            window_title_colour = BACKGROUND_COLOUR;
        }

        Framebuffer::fill_rect(
            Some(&self.clipped_rectangles),
            self.x,
            self.y,
            self.width,
            self.height,
            window_background_colour,
        );

        // Render window border
        Framebuffer::draw_rect_outline(
            Some(&self.clipped_rectangles),
            self.x,
            self.y,
            self.width,
            self.height,
            window_border_colour,
        );

        // Render window bar
        Framebuffer::fill_rect(
            Some(&self.clipped_rectangles),
            self.x + 3,
            self.y + 3,
            self.width - 3,
            20,
            window_title_colour,
        );

        // Render window bar bottom line
        Framebuffer::draw_horizontal_line(
            Some(&self.clipped_rectangles),
            self.x,
            self.y + 21,
            self.width,
            window_border_colour,
        );
    }

    // WARNING: May not work with multiple calls to paint/multiple windows
    // Directly punches out areas from rectangle and returns a list of rectangles which can be output upon the screen
    fn subtract_rectangle(&mut self, mut clipping_rect: Rectangle) {
        let mut subject = Rectangle::from_window(&self);

        if !(clipping_rect.left <= subject.right
            && clipping_rect.right >= subject.left
            && clipping_rect.top <= subject.bottom
            && clipping_rect.bottom >= subject.top)
        {
            return;
        }

        let split_rectangles = Rectangle::split(&mut subject, &mut clipping_rect);
        // let old_rect = self.clipped_rectangles.remove_at(i);
        // PAGE_FRAME_ALLOCATOR.lock().free_frame(old_rect as *mut u64);
        // PAGE_FRAME_ALLOCATOR.free();
        self.clipped_rectangles.head = split_rectangles.head;
    }

    // fn add_rectangle(&mut self, mut new_rectangle: Rectangle) {
    //     self.subtract_rectangle(new_rectangle);
    //     self.clipped_rectangles.push(
    //         PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
    //         new_rectangle,
    //     );
    //     PAGE_FRAME_ALLOCATOR.free();
    // }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    top: u64,
    bottom: u64,
    left: u64,
    right: u64,
}

impl Rectangle {
    pub fn new(top: u64, bottom: u64, left: u64, right: u64) -> Self {
        Rectangle {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
        }
    }

    pub fn from_window(window: &Window) -> Self {
        Self::new(
            window.y,
            window.y + window.height,
            window.x,
            window.x + window.width,
        )
    }

    /*
        Method is called upon the subject rect (bottom), given a clipping rect (top)
        Returns a list of rectangles that can be drawn by splitting subject by clipping upon various axes
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
            PAGE_FRAME_ALLOCATOR.free();
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
            PAGE_FRAME_ALLOCATOR.free();
        }

        // Check if clipping rect right side intersects with subject
        if clipping_rect.right > self.left && clipping_rect.right < self.right {
            let new_rect = Rectangle::new(self.top, self.bottom, self.left, clipping_rect.left);
            self.left = clipping_rect.left;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
            PAGE_FRAME_ALLOCATOR.free();
        }

        // Check if clipping rect bottom intersects with subject
        if clipping_rect.bottom > self.top && clipping_rect.bottom < self.bottom {
            let new_rect = Rectangle::new(self.top, clipping_rect.top, self.left, self.right);
            self.left = clipping_rect.left;

            split_rectangles.push(
                PAGE_FRAME_ALLOCATOR.lock().alloc_frame().unwrap() as u64,
                new_rect,
            );
            PAGE_FRAME_ALLOCATOR.free();
        }

        return split_rectangles;
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
    /*  Recives a list of clipped rectangles (these rectangles are not to be rendered upon the screen)
        Loops through that list and clamps the clipping rects to the window before drawing
        If clipped rectangles are not supplied, simply clamps to the screen and draws pixels
    */
    pub fn fill_rect(
        clipped_rectangles: Option<&Stack<Rectangle>>,
        mut x: u64,
        mut y: u64,
        width: u64,
        height: u64,
        colour: u32,
    ) {
        match clipped_rectangles {
            Some(rectangles) => {
                if rectangles.head.is_some() {
                    for (_i, clipped_rectangle) in rectangles.into_iter().enumerate() {
                        let clip = clipped_rectangle.unwrap().payload;

                        // Clamp of printable area to clipped region itself
                        let x_base = core::cmp::max(x, clip.left);
                        let y_base = core::cmp::max(y, clip.top);
                        let x_limit = core::cmp::min(x + width, clip.right);
                        let y_limit = core::cmp::min(y + height, clip.bottom);

                        for i in x_base..x_limit {
                            for j in y_base..y_limit {
                                Framebuffer::draw_pixel(i, j, colour);
                            }
                        }
                    }
                } else {
                    return Framebuffer::fill_rect(None, x, y, width, height, colour);
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
                            Framebuffer::draw_pixel(i, j, colour);
                        }
                    }
                }
            }
        }
    }

    // Draws outline of rectangle only
    pub fn draw_rect_outline(
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        colour: u32,
    ) {
        if x.checked_add(width).is_some() && y.checked_add(height).is_some() {
            Framebuffer::draw_horizontal_line(clipped_rectangles, x, y, width, colour);
            Framebuffer::draw_horizontal_line(clipped_rectangles, x, y + height, width, colour);

            Framebuffer::draw_vertical_line(clipped_rectangles, x, y, height, colour);
            Framebuffer::draw_vertical_line(clipped_rectangles, x + width, y, height, colour);
        }
    }

    pub fn draw_pixel(x: u64, y: u64, byte: u32) {
        unsafe {
            // TODO: make this use framebuffer array for safety
            let offset = (0x180000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
            *offset = byte;
        }
    }

    fn draw_horizontal_line(
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        length: u64,
        colour: u32,
    ) {
        Framebuffer::fill_rect(clipped_rectangles, x, y, length, 3, colour);
    }

    fn draw_vertical_line(
        clipped_rectangles: Option<&Stack<Rectangle>>,
        x: u64,
        y: u64,
        length: u64,
        colour: u32,
    ) {
        Framebuffer::fill_rect(clipped_rectangles, x, y, 3, length, colour);
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
        Framebuffer::fill_rect(None, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00);
    }

    fn put_char(&mut self, character: char) {
        match character {
            _ => self.draw_character(character),
        }
    }
}

pub fn init(framebuffer_tag: FramebufferTag) {
    // Setup font information
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;
    let _font = unsafe { &*(font_start as *const PsfFont) };

    FRAMEBUFFER.lock().pitch = framebuffer_tag.pitch as u64;
    FRAMEBUFFER.lock().bpp = framebuffer_tag.bpp as u64;

    // Map the framebuffer into accessible memory
    paging::identity_map_from(framebuffer_tag.address, 3);

    // Make background blank
    Framebuffer::fill_rect(None, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, BACKGROUND_COLOUR);
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
