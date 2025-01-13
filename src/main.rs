//! Demonstrates use of the Flipper Zero Dialog API.
//!
//! Creates a dialog with three buttons, and displays a message depending on which button was pressed.

#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocator
extern crate alloc;
extern crate flipperzero_alloc;

use core::ffi::{c_void, CStr};
use core::ptr;
use flipperzero::{format};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;
use flipperzero_sys::{furi_delay_ms, Align_AlignCenter, Align_AlignLeft, Align_AlignRight, Align_AlignTop, Font_FontBigNumbers, Font_FontSecondary, InputEvent, InputKey_InputKeyBack, InputKey_InputKeyDown, InputKey_InputKeyLeft, InputKey_InputKeyOk, InputKey_InputKeyRight, InputKey_InputKeyUp, InputType_InputTypePress};
use flipperzero_sys::furi::UnsafeRecord;

const FULLSCREEN: sys::GuiLayer = sys::GuiLayer_GuiLayerFullscreen;
const MAX_DICE: u8 = 99;
const POINTER: &CStr = c">";

manifest!(name = "Dice");
entry!(main);

/// View draw handler.
pub unsafe extern "C" fn draw_callback(canvas: *mut sys::Canvas, context: *mut c_void) {
    unsafe {
        let app = context as *mut App;

        // If the "rolled" number is non-zero, display it
        if (*app).rolled > 0 {
            // display rolled values as Big Numbers
            sys::canvas_set_font(canvas, Font_FontBigNumbers);

            // Build the string version of the rolled number
            let text = format!("{}", (*app).rolled);

            // Do math to figure out how to center the text
            let canvas_width = sys::canvas_width(canvas);
            let text_width = sys::canvas_string_width(canvas, text.as_c_ptr());
            let x_offset = (canvas_width as i32 - text_width as i32) / 2;

            let canvas_height = sys::canvas_height(canvas);
            let y_offset = canvas_height as i32 / 2;

            // draw the rolled number
            sys::canvas_draw_str_aligned(canvas, x_offset, y_offset, Align_AlignLeft,
                                         Align_AlignCenter, text.as_c_ptr());
        } else {
            // if the rolled number is 0, we're picking which dice to roll
            
            // set the font size to the small one
            sys::canvas_set_font(canvas, Font_FontSecondary);

            // for each type of dice
            for i in 0..6usize {
                let y_offset = i as i32 * 10;

                // if this dice-type is selected, draw the pointer
                if (*app).selected == i {
                    sys::canvas_draw_str_aligned(canvas, 0, y_offset, Align_AlignLeft,
                                                 Align_AlignTop, POINTER.as_ptr());
                }

                // draw the dice-count as right-aligned, so the d's all line up
                // (the font is not fixed-width)
                let count_text = format!("{}", (*app).counts[i]);
                sys::canvas_draw_str_aligned(canvas, 18, y_offset, Align_AlignRight,
                                             Align_AlignTop, count_text.as_c_ptr());
                
                sys::canvas_draw_str_aligned(canvas, 19, y_offset, Align_AlignLeft,
                                             Align_AlignTop, DICE_NAMES[i].as_ptr());
            }
        }
    }
}
#[allow(non_upper_case_globals)]
pub unsafe extern "C" fn input_callback(input_event: *mut InputEvent, context: *mut c_void) {
    unsafe {
        let app = context as *mut App;
        let key = (*input_event).key;

        if (*input_event).type_ != InputType_InputTypePress {
            return;
        }


        if (*app).rolled > 0 && (key == InputKey_InputKeyOk || key == InputKey_InputKeyBack) {
            (*app).rolled = 0;
            return;
        }

        match key {
            InputKey_InputKeyUp => {
                (*app).selected = ((*app).selected + 6 - 1) % 6;
            },
            InputKey_InputKeyDown => {
                (*app).selected = ((*app).selected + 1) % 6;
            },
            InputKey_InputKeyLeft => {
                let mut new_count = (*app).counts[(*app).selected];
                if new_count > 0 {
                    new_count -= 1;
                }
                (*app).counts[(*app).selected] = new_count;
            },
            InputKey_InputKeyRight => {
                let mut new_count = (*app).counts[(*app).selected];
                if new_count < MAX_DICE {
                    new_count += 1;
                }
                (*app).counts[(*app).selected] = new_count;
            },
            InputKey_InputKeyBack => {
                (*app).quit = true;
            },
            InputKey_InputKeyOk => {
                let mut total: u32 = 0;
                for i in 0..6usize {
                    for _ in 0..(*app).counts[i] {
                        total += (sys::furi_hal_random_get() % DICE_SIZES[i]) + 1;
                    }
                }
                (*app).rolled = total;
            }
            _ => {}
        }
    }
}

const DICE_NAMES: [&CStr; 6] = [c"d20", c"d12", c"d10", c"d8", c"d6", c"d4"];
const DICE_SIZES: [u32; 6] = [20, 12, 10, 8, 6, 4];

struct App {
    selected: usize,
    counts: [u8; 6],
    quit: bool,
    rolled: u32
}

impl App {
    pub fn new() -> Self {
        App {
            selected: 0,
            counts: [0, 0, 0, 0, 0, 0],
            quit: false,
            rolled: 0,
        }
    }
}

fn main(_args: Option<&CStr>) -> i32 {
    let mut app = App::new();
    unsafe {
        let view_port = sys::view_port_alloc();
        let context_pointer = ptr::from_mut(&mut app) as *mut c_void;
        sys::view_port_input_callback_set(
            view_port,
            Some(input_callback),
            context_pointer
        );
        sys::view_port_draw_callback_set(
            view_port,
            Some(draw_callback),
            context_pointer
        );

        {
            let gui = UnsafeRecord::open(c"gui".as_ptr());
            sys::gui_add_view_port(gui.as_ptr(), view_port, FULLSCREEN);
            loop {
                if app.quit {
                    break;
                }
                furi_delay_ms(500);
            }

            sys::view_port_enabled_set(view_port, false);
            sys::gui_remove_view_port(gui.as_ptr(), view_port);
        }
        sys::view_port_free(view_port);
    }

    0
}
