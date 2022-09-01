// src/string.rs

/*
    File contains custom operations meant to be used upon strings
*/

use core::str::from_utf8;

// Calculates length by checking for a blank character
fn strlen(mut string: *const u8) -> usize {
    let mut count = 0;
    loop {
        count += 1;
        unsafe {
            if *string == 0 {
                return count as usize;
            }
            string = string.offset(1);
        }
    }
}

// Converts a *const u8 pointer into a string which can be displayed
pub fn get_string_from_ptr(mut string_ptr: *const u8) -> &'static str {
    let len = strlen(string_ptr);
    let string_array = unsafe { core::slice::from_raw_parts(string_ptr, len) };
    from_utf8(string_array).unwrap().trim()
}
