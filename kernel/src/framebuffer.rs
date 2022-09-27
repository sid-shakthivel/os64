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
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16

    Utilises double buffering of sorts
    Each window contains a buffer of it's internal state in which work is completed upon
    The frontbuffer is written to through each window buffer
    Advantage is users do not see pixel modifications and writting to video memory is expensive
*/

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::list::Stack;
use crate::page_frame_allocator::{FrameAllocator, PAGE_FRAME_ALLOCATOR};
use crate::print_serial;
use crate::spinlock::Lock;
use crate::writer::Writer;
use crate::CONSOLE;
use crate::{page_frame_allocator, paging};
use bitflags::bitflags;
use multiboot2::FramebufferTag;

pub const SCREEN_WIDTH: u64 = 1024;
pub const SCREEN_HEIGHT: u64 = 768;
pub const BACKGROUND_COLOUR: u32 = 0x0b0554;
pub const WINDOW_BACKGROUND_COLOUR: u32 = 0xc6d0ff;
pub const WINDOW_TITLE_COLOUR: u32 = 0x00b5da;
pub const WINDOW_TITLE_HEIGHT: u64 = 20;

/*
    This struct is used by processes to encapsulate information that a user program may require
    Allows usermode process to access mouse, keyboard data
*/
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C, packed)]
pub struct Event {
    mouse_x: i32,
    mouse_y: i32,
    scancode: i32,
    mask: i32,
    key_pressed: u8,
}

bitflags! {
    pub struct EventFlags: i32 {
        const KEY_PRESSED = 0b00000001;
        const MOUSE_UPDATED = 0b00000010;
        const MOUSE_CLICKED = 0b00000100;
    }
}

impl Event {
    pub const fn new(
        mask: i32,
        key_pressed: u8,
        scancode: u8,
        mouse_x: u64,
        mouse_y: u64,
    ) -> Event {
        Event {
            mouse_x: mouse_x as i32,
            mouse_y: mouse_y as i32,
            scancode: scancode as i32,
            mask,
            key_pressed,
        }
    }
}

pub static mut EVENT_MEMORY_LOCATION: *mut Event = 0 as *mut Event;

pub static WINDOW_MANAGER: Lock<WindowManager> = Lock::new(WindowManager::new(1024, 768));

pub trait FramebuffferEntity {
    fn clip(&mut self, dirty_rectangles: Stack<Rectangle>) {} // Performs clipping
    fn refresh(&mut self) {} // Writes pixels from buffer to framebuffer essentially refreshing the screen
    fn clipped_rectangles(&mut self) -> &mut Stack<Rectangle>; // Returns a reference to clipped rectangles

    /*
        Subtracts regions of an rectangle from another (punches out regions)
        Loops over all previous clips to ensure everything works
    */
    fn subtract_rectangle(&mut self, clipping_rect: &Rectangle) {
        // Loop through the clipping rects
        let mut index = 0;

        while index < self.clipped_rectangles().length {
            let mut raw = self.clipped_rectangles().get_at(index);

            // If the clipping rect intersects with a rectangle (subject) split it or else move onto the next one
            if clipping_rect.left < raw.right
                && clipping_rect.right > raw.left
                && clipping_rect.top < raw.bottom
                && clipping_rect.bottom > raw.top
            {
                // Remove old rectangle and replace with list of rects
                self.clipped_rectangles().append(raw.split(clipping_rect));
                self.clipped_rectangles().remove_at(index);

                // Reset the counter
                index = 0;
            } else {
                index += 1;
            }
        }
    }

    // Subtracts regions of rectangles from subject, but includes the new rectangle
    fn add_rectangle(&mut self, clipping_rect: &Rectangle) {
        self.subtract_rectangle(clipping_rect);
        self.clipped_rectangles().push(*clipping_rect);
    }
}

/*
    Window manager contains all other windows and orchestrates their actions including handling mouse, keyboard
    Handles the general desktop/background
    Writes directly to framebuffer
*/
#[derive(Debug, Clone, PartialEq)]
pub struct WindowManager {
    width: u64,
    height: u64,
    clipped_rectangles: Stack<Rectangle>,
    pub child_windows: Stack<Window>,
    drag_child: Option<*mut Window>,
    mouse_x: u64,
    mouse_y: u64,
    drag_x_offset: u64,
    drag_y_offset: u64,
    wid_counter: u64,
}

impl WindowManager {
    pub const fn new(width: u64, height: u64) -> WindowManager {
        WindowManager {
            width,
            height,
            clipped_rectangles: Stack::<Rectangle>::new(),
            child_windows: Stack::<Window>::new(),
            drag_child: None,
            mouse_x: SCREEN_WIDTH / 2,
            mouse_y: SCREEN_HEIGHT / 2,
            drag_x_offset: 0,
            drag_y_offset: 0,
            wid_counter: 0,
        }
    }

    pub fn paint(&mut self, dirty_rectangles: Stack<Rectangle>, paint_children: bool) {
        // Apply bound clipping to obtain visible parts of window
        self.clip(dirty_rectangles.clone());

        // Actually draw window by copying to framebuffer
        self.refresh();

        // Clear clipping rects
        self.clipped_rectangles.empty();

        // Optionally repaint children
        if paint_children {
            for (_i, child) in self.child_windows.into_iter().enumerate() {
                child
                    .unwrap()
                    .payload
                    .clone()
                    .paint(dirty_rectangles.clone());
            }
        }
    }

    // Creates a new window within itself (utilised for the desktop window)
    pub fn add_sub_window(&mut self, window: &mut Window) -> u64 {
        window.wid = self.wid_counter;
        self.wid_counter += 1;
        self.child_windows.push(window.clone());
        window.wid
    }

    // Updates the event upon a selected window being sent for a process
    pub fn handle_event(&mut self) -> Option<*mut Event> {
        // Get the top child as that is likely selected by user
        if let Some(selected_window_wrapped) = self.child_windows.head {
            // Clone the event and write it's contents to the event memory locatory
            let selected_window = unsafe { (*selected_window_wrapped).payload.clone() };
            let cloned_event = selected_window.event.clone();

            unsafe {
                core::ptr::write_bytes(EVENT_MEMORY_LOCATION as *mut u8, 0, 512);
                *EVENT_MEMORY_LOCATION = cloned_event;

                // Remove flags for event after clone has been made to prevent duplicates
                (*selected_window_wrapped).payload.event.mask &= !0b00000001; // Remove key pressed flag
                (*selected_window_wrapped).payload.event.mask &= !0b00000010; // Remove mouse updated flag

                return Some(EVENT_MEMORY_LOCATION);
            }
        }
        return None;
    }

    // Updates event field upon a selected window on a key click
    pub fn handle_keyboard(&mut self, key_pressed: char, scancode: u8) {
        // Get the top child as that is likely selected by user
        if let Some(selected_window_wrapped) = self.child_windows.head {
            // Ensure when handling updates it sets the correct mask and key
            unsafe {
                (*selected_window_wrapped).payload.event.key_pressed = key_pressed as u8;
                (*selected_window_wrapped).payload.event.scancode = scancode as i32;
                (*selected_window_wrapped).payload.event.mask |= 0b00000001;
            }
        }
    }

    /*
        Loops through windows to find window in which mouse coordinates are within
        Returns both the index of the window along with the window itself
    */
    fn get_selected_child(&mut self, mouse_x: u64, mouse_y: u64) -> (usize, Option<Window>) {
        for (i, window) in self.child_windows.into_iter().enumerate() {
            let temp_window = window.unwrap().payload.clone();
            if mouse_x >= temp_window.x
                && mouse_x <= (temp_window.x + temp_window.width)
                && mouse_y >= temp_window.y
                && mouse_y <= (temp_window.y + temp_window.height)
            {
                // Update drag window, etc
                let const_ptr = &window.unwrap().payload as *const Window;
                let mut_ptr = const_ptr as *mut Window;
                self.drag_child = Some(mut_ptr);
                self.drag_x_offset = mouse_x - temp_window.x;
                self.drag_y_offset = mouse_y - temp_window.y;
                return (i, Some(temp_window));
            }
        }

        return (0, None);
    }

    pub fn handle_mouse(&mut self, mouse_x: u64, mouse_y: u64, is_left_pressed: bool) {
        // Check if mouse is actually pressed
        if is_left_pressed {
            // Obtain window which mouse is over
            let (index, wrapped_window) = self.get_selected_child(self.mouse_x, self.mouse_y);

            // Raise window
            if let Some(mut window) = wrapped_window {
                // window.raise(index);
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

        // Update mouse coordinates for event object
        if let Some(selected_window_wrapped) = self.child_windows.head {
            // Ensure when handling updates it sets the correct mask and key
            unsafe {
                (*selected_window_wrapped).payload.event.mouse_x = mouse_x as i32;
                (*selected_window_wrapped).payload.event.mouse_y = mouse_y as i32;
                (*selected_window_wrapped).payload.event.mouse_x |= 0b00000010;
            }
        }

        let mouse_rect = Rectangle::new(
            self.mouse_x,
            self.mouse_y,
            self.mouse_x + 5,
            self.mouse_y + 5,
        );
        let mut dirty_rectangles = Stack::<Rectangle>::new();
        dirty_rectangles.push(mouse_rect);

        // Repaint
        self.paint(dirty_rectangles, true);

        // Paint mouse
        for y in mouse_y..(mouse_y + 5) {
            for x in (mouse_x)..(mouse_x + 5) {
                let offset = (0xf50000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
                unsafe {
                    *offset = 0xFF0000;
                }
            }
        }

        self.mouse_x = mouse_x;
        self.mouse_y = mouse_y;
    }
}

impl FramebuffferEntity for WindowManager {
    // Applies clipping to a window against dirty rectangles, windows, titlebar, children
    fn clip(&mut self, dirty_rectangles: Stack<Rectangle>) {
        let subject_rect = Rectangle {
            top: 0,
            bottom: self.height,
            left: 0,
            right: self.width,
        };

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

            // Clip against children
            for (_i, child) in self.child_windows.clone().into_iter().enumerate() {
                let child_rectangle = Rectangle::from_window(&child.unwrap().payload);
                self.subtract_rectangle(&child_rectangle);
            }
        }
    }

    fn refresh(&mut self) {
        // Write pixels to the framebuffer for the background

        for (_i, clipped_rectangle) in self.clipped_rectangles.into_iter().enumerate() {
            let clip = clipped_rectangle.unwrap().payload;

            // Clamp printable area to clipped region itself
            let x_base = core::cmp::max(0, clip.left);
            let y_base = core::cmp::max(0, clip.top);
            let x_limit = core::cmp::min(self.width, clip.right);
            let y_limit = core::cmp::min(self.height, clip.bottom);

            for y in y_base..y_limit {
                for x in x_base..x_limit {
                    let offset = (0xf50000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
                    unsafe {
                        *offset = BACKGROUND_COLOUR;
                    }
                }
            }
        }
    }

    fn clipped_rectangles(&mut self) -> &mut Stack<Rectangle> {
        return &mut self.clipped_rectangles;
    }
}

/*
    Windows are poritions of a screen required by a process for a certain task
    Include data on coordinates, children, contents (buffer in which everything is written within), mouse, own custom events
*/

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub title: &'static str,
    pub x: u64,
    pub y: u64,
    pub width: u64,
    pub height: u64,
    colour: u32,
    clipped_rectangles: Stack<Rectangle>,
    event: Event,
    buffer: u64,
    parent: Option<*mut WindowManager>,
    pub wid: u64,
}

impl Window {
    pub fn new(
        title: &'static str,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        parent: Option<*mut WindowManager>,
        colour: u32,
    ) -> Self {
        let buffer_address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(350) as u64;
        PAGE_FRAME_ALLOCATOR.free();
        Window {
            title,
            x,
            y,
            width,
            height,
            colour,
            parent,
            clipped_rectangles: Stack::<Rectangle>::new(),
            event: Event::new(0, 0, 0, 0, 0),
            buffer: buffer_address,
            wid: 0,
        }
    }

    /*
        Paints window upon it's internal buffer
        Note: Does not modify actual framebuffer
    */
    pub fn paint(&mut self, dirty_rectangles: Stack<Rectangle>) {
        // Apply bound clipping to obtain visible parts of window
        self.clip(dirty_rectangles.clone());

        // Draw upon the screen
        self.refresh();

        // Clear clipping rects
        self.clipped_rectangles.empty();
    }

    /*
        Contents of window which is stored within bufer is what is copied to framebuffer upon a redraw
        Takes a y offset
        Copies bytes from another bufer to this internal buffer (works well for games like doom)
    */
    pub fn update_buffer_from_buffer(&mut self, buffer_src: *const u32, y_offset: u64) {
        let buffer_dest = self.buffer as *mut u32;

        let mut buffer_y = 0;

        for y in y_offset..self.height {
            for x in 0..self.width {
                unsafe {
                    *buffer_dest.offset((y * self.width + x) as isize) =
                        *buffer_src.offset((buffer_y * self.width + x) as isize);
                }
            }
            buffer_y += 1;
        }
    }

    /*
        Contents of window which is stored within bufer is what is copied to framebuffer upon a redraw
        Update region of the buffer by writting pixels of colour to a certain area
    */
    pub fn update_buffer_region_to_colour(
        &mut self,
        x_base: u64,
        x_limit: u64,
        y_base: u64,
        y_limit: u64,
        colour: u32,
    ) {
        let buffer_p = self.buffer as *mut u32;

        for y in y_base..y_limit {
            for x in x_base..x_limit {
                unsafe {
                    *buffer_p.offset((y * self.width + x) as isize) = colour;
                }
            }
        }
    }

    pub fn draw_string(&mut self, string: &str, mut x_base: u64, y_base: u64) {
        let buffer_p = self.buffer as *mut u32;

        for character in string.as_bytes() {
            unsafe {
                if let Some(font) = FONT {
                    let glyph_address = (FONT_START
                        + font.header_size
                        + (font.bytes_per_glyph * (character.clone() as u32)))
                        as *mut u8;

                    for cy in 0..16 {
                        let mut index = 8;
                        for cx in 0..8 {
                            let adjusted_x = cx + x_base;
                            let adjusted_y = cy + y_base;

                            // Load correct bitmap for glyph
                            let glyph_offset: u16 =
                                (*glyph_address.offset(cy as isize) as u16) & (1 << index);
                            if glyph_offset > 0 {
                                *buffer_p.offset((adjusted_y * self.width + adjusted_x) as isize) =
                                    0x01;
                            }
                            index -= 1;
                        }
                    }

                    x_base += 8;
                }
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
                    (*parent).paint(dirty_rectangles.clone(), true);

                    // Repaint windows below the moving window
                    let windows_below = (*parent).child_windows.get_lower_nodes(self.clone());
                    for (_i, window) in windows_below.into_iter().enumerate() {
                        window
                            .unwrap()
                            .payload
                            .clone()
                            .paint(dirty_rectangles.clone());
                    }
                    // windows_below.empty();
                }
            }
        }

        self.paint(Stack::<Rectangle>::new());
    }

    /*
        Subtracts regions of an rectangle from another (punches out regions)
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

    // Subtracts regions of rectangles from subject, but includes the new rectangle
    fn add_rectangle(&mut self, clipping_rect: &Rectangle) {
        self.subtract_rectangle(clipping_rect);
        self.clipped_rectangles.push(*clipping_rect);
    }
}

impl FramebuffferEntity for Window {
    // Applies clipping to a window against dirty rectangles, self, windows above
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

            // Get windows above
            if let Some(parent) = self.parent {
                let windows_above =
                    unsafe { (*parent).child_windows.get_higher_nodes(self.clone()) };

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

    fn refresh(&mut self) {
        // Write pixels from internal window buffer to the framebuffer

        for (_i, clipped_rectangle) in self.clipped_rectangles.into_iter().enumerate() {
            let clip = clipped_rectangle.unwrap().payload;

            // Check whether the clipped rect is within the window
            if clip.top >= self.y
                && clip.bottom <= (self.y + self.height)
                && clip.left >= self.x
                && clip.right <= (self.x + self.width)
            {
                // Copy each clipping rectangle to the framebuffer

                let x_base = clip.left;
                let y_base = clip.top;
                let x_limit = clip.right;
                let y_limit = clip.bottom;

                let mut buffer_x = x_base - self.x;
                let mut buffer_y = y_base - self.y;

                for y in y_base..y_limit {
                    for x in x_base..x_limit {
                        let offset = (0xf50000 + (y * 4096) + ((x * 32) / 8)) as *mut u32;
                        let buffer_p = self.buffer as *const u32;
                        unsafe {
                            *offset = *buffer_p.offset((buffer_y * self.width + buffer_x) as isize)
                        }
                        buffer_x += 1;
                    }
                    buffer_x = 0;
                    buffer_y += 1;
                }
            }
        }
    }

    fn clipped_rectangles(&mut self) -> &mut Stack<Rectangle> {
        &mut self.clipped_rectangles
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

#[derive(Copy, Clone, Debug)]
struct PsfFont {
    magic: u32,
    version: u32,         // Usually 0
    header_size: u32,     // Offset of bitmaps
    flags: u32,           // 0 If there isn't a unicode table
    glymph_num: u32,      // Number of glyghs
    bytes_per_glyph: u32, // Size of each glygh
    height: u32,          // In pixels
    width: u32,           // In pixels
}

impl PsfFont {
    pub fn verify(&self) {
        assert!(
            self.magic == 0x864ab572,
            "PsfFont magic is not {}",
            0x864ab572 as u32
        );

        assert!(self.version == 0, "PsfFont version is not 0");

        assert!(
            self.bytes_per_glyph == 16,
            "PsfFont bytes per glyph is not 16"
        );

        assert!(self.height == 16, "PsfFont is not 16");

        assert!(self.width == 8, "PsfFont is not 8");
    }
}

impl Writer for WindowManager {
    fn clear(&mut self) {
        // self.fill_rect(None, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 0x00);
    }

    fn put_char(&mut self, character: char) {
        // match character {
        // _ => self.draw_clipped_character(None, character, 0, 0, 0, 0),
        // }
    }
}

static mut FONT: Option<PsfFont> = None;
static mut FONT_START: u32 = 0;

pub fn init(framebuffer_tag: FramebufferTag) {
    // Setup font
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;

    unsafe {
        FONT = Some(*(font_start as *const PsfFont));
        FONT_START = font_start;
        FONT.unwrap().verify();
    }

    // Calulate sizes of the framebuffer
    let size_in_bytes = ((framebuffer_tag.bpp as u64)
        * (framebuffer_tag.width as u64)
        * (framebuffer_tag.height as u64))
        / 8;

    let size_in_mb = page_frame_allocator::convert_bytes_to_mb(size_in_bytes);

    let pages_required = page_frame_allocator::get_page_number(
        page_frame_allocator::round_to_nearest_page(size_in_bytes),
    );

    // Setup the front buffer
    let frontbuffer_address = PAGE_FRAME_ALLOCATOR.lock().alloc_frames(pages_required) as u64;
    PAGE_FRAME_ALLOCATOR.free();

    // Identity map this buffer so it maps to video memory
    paging::identity_map_from(framebuffer_tag.address, frontbuffer_address, size_in_mb);

    print_serial!("FB ADDRESS 0x{:x}\n", frontbuffer_address);

    unsafe {
        EVENT_MEMORY_LOCATION = PAGE_FRAME_ALLOCATOR.lock().alloc_frame() as *mut Event;
        PAGE_FRAME_ALLOCATOR.free();
    }
}

// pub fn fill_gradient_rect(&mut self, x: u64, y: u64, width: u64, height: u64) {
//     let r1 = ((0x00b5da & 0xFF0000) >> 16) as f64;
//     let r2 = ((0x0b0554 & 0xFF0000) >> 16) as f64;

//     let g1 = ((0x00b5da & 0x00FF00) >> 8) as f64;
//     let g2 = ((0x0b0554 & 0x00FF00) >> 8) as f64;

//     let b1 = (0x00b5da & 0x0000FF) as f64;
//     let b2 = (0x0b0554 & 0x0000FF) as f64;

//     let h = (height) as f64;

//     let r_offset: f64 = (r2 - r1) / h;
//     let g_offset: f64 = (g2 - g1) / h;
//     let b_offset: f64 = (b2 - b1) / h;

//     let mut current_r = r1;
//     let mut current_g = g1;
//     let mut current_b = b1;

//     let mut colour = BACKGROUND_COLOUR;

//     for i in y..(y + height) {
//         colour = current_b as u32 | ((current_g as u32) << 8) | ((current_r as u32) << 16);
//         for j in x..(x + width) {
//             self.draw_pixel(j, i, colour);
//         }
//         current_r += r_offset;
//         current_g += g_offset;
//         current_b += b_offset;
//     }
// }

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
