// Copyright Jeron A. Lau 2017-2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use std::ffi::c_void;
use std::ptr::null_mut;

pub struct Window {
    // Keyboard (XKB)
    keymap: *mut c_void,
    context: *mut c_void,
    state: *mut c_void,
    compose: *mut c_void,
    xkb: XkbCommonX11,
    // Window (XCB)
    pub(crate) window: u32,
    pub(crate) connection: *mut c_void,
    wh: (u16, u16),
    xcb: Xcb,
}

impl Window {
    pub fn new(v: Option<i32>) -> Self {
        let (xcb, xkb) = xcb_load();
        let connection = xcb_connect(&xcb);
        let mut screen = xcb_screen(connection, &xcb);
        let window = xcb_window(connection, &xcb, &mut screen, v);
        let (state, keymap, context, compose) = xkb_keyboard(&xkb);
        let wh = (screen.width_in_pixels, screen.height_in_pixels);

        Window {
            keymap,
            context,
            state,
            compose,
            xkb,
            window,
            connection,
            wh,
            xcb,
        }
    }

    pub(crate) fn update(
        &mut self,
    ) -> bool {
        let mut rtn = false;
        crate::hid::set_char(0, '\0');
        unsafe { (self.xcb.xcb_flush)(self.connection) };
        while xcb_poll_for_event(
            self.connection,
            &self.xcb,
            &self.xkb,
            self.state,
            self.compose,
            &mut self.wh,
            &mut rtn,
        ) { }
        rtn
    }

    pub fn wh(&self) -> (u16, u16) {
        self.wh
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            (self.xkb.xkb_state_unref)(self.state);
            (self.xkb.xkb_keymap_unref)(self.keymap);
            (self.xkb.xkb_context_unref)(self.context);
            (self.xcb.xcb_destroy_window)(self.connection, self.window);
            (self.xcb.xcb_disconnect)(self.connection);
        }
    }
}

dl_api!(Xcb, "libxcb.so.1",
	fn xcb_poll_for_event(*mut c_void) -> *mut XcbGenericEvent,
	fn xcb_flush(*mut c_void) -> i32,
	fn xcb_intern_atom(*mut c_void, u8, u16, *const u8) -> u32,
	fn xcb_intern_atom_reply(*mut c_void, u32, *mut c_void)
		-> *mut XcbInternAtomReply,
	fn xcb_change_property(*mut c_void, u8, u32, u32, u32, u8, u32,
		*const c_void) -> u32,
	fn xcb_map_window(*mut c_void, u32) -> u32,
	fn xcb_get_setup(*mut c_void) -> *mut c_void,
	fn xcb_setup_roots_iterator(*mut c_void) -> XcbScreenIterator,
	fn xcb_generate_id(*mut c_void) -> u32,
	fn xcb_create_window(*mut c_void, u8, u32, u32, i16, i16, u16, u16, u16,
		u16, u32, u32, *mut u32) -> u32,
	fn xcb_connect(*mut c_void, *mut c_void) -> *mut c_void,
	fn xcb_destroy_window(*mut c_void, u32) -> u32,
	fn xcb_disconnect(*mut c_void) -> ()
);

dl_api!(XkbCommonX11, "libxkbcommon-x11.so.0",
	fn xkb_context_unref(*mut c_void) -> (),
	fn xkb_keymap_unref(*mut c_void) -> (),
	fn xkb_state_unref(*mut c_void) -> (),
	fn xcb_xkb_use_extension(*mut c_void, u16, u16) -> u32,
	fn xkb_state_key_get_utf8(*mut c_void, u32, *mut u8, usize) -> i32,
	fn xkb_state_update_key(*mut c_void, u32, KeyDirection)
		-> StateComponent,
	fn xkb_x11_state_new_from_device(*mut c_void, *mut c_void, i32)
		-> *mut c_void,
    fn xkb_state_new(*mut c_void) -> *mut c_void,
	fn xkb_x11_keymap_new_from_device(*mut c_void, *mut c_void, i32,
		CompileFlags) -> *mut c_void,
    fn xkb_keymap_new_from_names(*mut c_void, *const c_void, CompileFlags) -> *mut c_void,
    fn xkb_keymap_new_from_string(*mut c_void, *const i8, u32, CompileFlags) -> *mut c_void,
	fn xkb_context_new(ContextFlags) -> *mut c_void,
	fn xkb_x11_get_core_keyboard_device_id(*mut c_void) -> i32,
    fn xkb_compose_table_new_from_locale(*mut c_void, *const i8, u32) -> *mut c_void,
    fn xkb_compose_state_new(*mut c_void, u32) -> *mut c_void,
    fn xkb_compose_state_get_utf8(*mut c_void, *mut u8, usize) -> i32,
    fn xkb_compose_state_get_status(*mut c_void) -> XkbComposeStatus,
    fn xkb_compose_state_feed(*mut c_void, u32) -> u32,
    fn xkb_state_key_get_one_sym(*mut c_void, u32) -> u32
);

#[allow(dead_code)]
#[repr(C)]
enum XkbComposeStatus {
    Nothing,
    Composing,
    Composed,
    Cancelled,
}

#[allow(dead_code)]
#[repr(C)]
enum StateComponent {
    None,
}

#[repr(C)]
enum KeyDirection {
    Up,
    Down,
}

#[repr(C)]
struct XcbInternAtomReply {
    response_type: u8,
    pad0: u8,
    sequence: u16,
    length: u32,
    atom: u32,
}

#[repr(C)]
enum CompileFlags {
    NoFlags = 0,
}

#[repr(C)]
enum ContextFlags {
    NoFlags = 0,
}

#[repr(C)]
#[derive(Clone)]
struct XcbScreen {
    root: u32,
    default_colormap: u32,
    white_pixel: u32,
    black_pixel: u32,
    current_input_masks: u32,
    width_in_pixels: u16,
    height_in_pixels: u16,
    width_in_millimeters: u16,
    height_in_millimeters: u16,
    min_installed_maps: u16,
    max_installed_maps: u16,
    root_visual: u32,
    backing_stores: u8,
    save_unders: u8,
    root_depth: u8,
    allowed_depths_len: u8,
}

#[repr(C)]
struct XcbScreenIterator {
    data: *mut XcbScreen,
    rem: i32,
    index: i32,
}

#[repr(C)]
#[derive(Clone)]
struct XcbGenericEvent {
    response_type: u8,
    detail: u8,
    sequence: u16,
    timestamp: u32,
    root: u32,
    event: u32,
    child: u32,
    root_x: i16,
    root_y: i16,
    event_x: i16,
    event_y: i16,
    state: u16,
    same_screen: u8,
    pad0: u8,
}

fn xcb_load() -> (Xcb, XkbCommonX11) {
    unsafe fn load_xcb_dl() -> Result<(Xcb, XkbCommonX11), ::dl_api::Error> {
        Ok((Xcb::new()?, XkbCommonX11::new()?))
    }
    unsafe { load_xcb_dl() }.unwrap_or_else(|err| {
        eprintln!("ERROR: couldn't find XCB: \"{}\", aborting...", err);
        ::std::process::abort();
    })
}

fn xcb_connect(xcb: &Xcb) -> *mut c_void {
    let connection = unsafe { (xcb.xcb_connect)(null_mut(), null_mut()) };
    if connection.is_null() {
        eprintln!("ERROR: XCB couldn't connect to X server, aborting...");
        ::std::process::abort();
    }
    connection
}

fn xcb_screen(connection: *mut c_void, xcb: &Xcb) -> XcbScreen {
    let setup = unsafe { (xcb.xcb_get_setup)(connection) };
    unsafe { (*((xcb.xcb_setup_roots_iterator)(setup).data)).clone() }
}

fn xcb_window(connection: *mut c_void, xcb: &Xcb, screen: &mut XcbScreen, v: Option<i32>) -> u32 {
    let atom1 = get_atom(connection, xcb, b"_MOTIF_WM_HINTS");
    let atom2 = get_atom(connection, xcb, b"_NET_WM_STATE");
    let atom3 = get_atom(connection, xcb, b"_NET_WM_STATE_MAXIMIZED_VERT");
    let atom4 = get_atom(connection, xcb, b"_NET_WM_STATE_MAXIMIZED_HORZ");
    let atom5 = get_atom(connection, xcb, b"WM_PROTOCOLS");
    let atom6 = get_atom(connection, xcb, b"WM_DELETE_WINDOW");
    let window = unsafe { (xcb.xcb_generate_id)(connection) };
    let mut value_list = [0b01000100000000001101111];
    if let Some(v) = v {
        screen.root_visual = unsafe { ::std::mem::transmute(v) };
    }
    unsafe {
        (xcb.xcb_create_window)(
            connection,
            0,
            window,
            screen.root,
            0,
            0,
            screen.width_in_pixels,
            screen.height_in_pixels,
            0,
            1,
            screen.root_visual,
            2048,
            &mut value_list[0],
        );
        (xcb.xcb_change_property)(
            connection,
            0,
            window,
            atom1,
            atom1,
            32,
            5,
            &[2u32, 0, 0, 0, 0] as *const _ as *const c_void,
        );
        (xcb.xcb_change_property)(
            connection,
            0,
            window,
            atom2,
            4,
            32,
            2,
            [atom3, atom4].as_ptr() as *const _ as *const c_void,
        );
        (xcb.xcb_change_property)(
            connection,
            0,
            window,
            atom5,
            4,
            32,
            1,
            [atom6].as_ptr() as *const _ as *const c_void,
        );
        (xcb.xcb_map_window)(connection, window);
        (xcb.xcb_flush)(connection);
    }
    window
}

fn get_atom(connection: *mut c_void, xcb: &Xcb, name: &[u8]) -> u32 {
    let atom = unsafe { (xcb.xcb_intern_atom)(connection, 0, name.len() as u16, &name[0]) };
    let reply = unsafe { (xcb.xcb_intern_atom_reply)(connection, atom, null_mut()) };
    let atom = unsafe {
        extern "C" {
            fn free(this: *mut XcbInternAtomReply) -> ();
        }
        let r_atom = (*reply).atom;
        free(reply);
        r_atom
    };
    atom
}

fn xkb_keyboard(
    xkb: &XkbCommonX11,
) -> (*mut c_void, *mut c_void, *mut c_void, *mut c_void) {
    use std::process::Command;

    let locale = std::ffi::CString::new(match std::env::var("LC_ALL") {
        Ok(val) => val,
        Err(_) => match std::env::var("LC_CTYPE") {
            Ok(val) => val,
            Err(_) => match std::env::var("LANG") {
                Ok(val) => val,
                Err(_) => "C".to_string(),
            },
        },
    }).unwrap();

    Command::new("xkbcomp")
        .arg("-xkb")
        .arg("$DISPLAY")
        .arg("/tmp/xkbmap")
        .output()
        .expect("failed to execute process");

    let string = std::ffi::CString::new(std::fs::read_to_string("/tmp/xkbmap").expect("oops")).unwrap();

    let context = unsafe { (xkb.xkb_context_new)(ContextFlags::NoFlags) };
    let keymap = unsafe {
        (xkb.xkb_keymap_new_from_string)(context, string.as_ptr(), 1, CompileFlags::NoFlags)
    };
    let state = unsafe { (xkb.xkb_state_new)(keymap) };
    let compose_table = unsafe { (xkb.xkb_compose_table_new_from_locale)(context, locale.as_ptr(), 0) };
    let compose = unsafe { (xkb.xkb_compose_state_new)(compose_table, 0) };

    (state, keymap, context, compose)
}

fn xcb_poll_for_event(
    connection: *mut c_void,
    xcb: &Xcb,
    xkb: &XkbCommonX11,
    state: *mut c_void,
    state2: *mut c_void,
    wh: &mut (u16, u16),
    rtn: &mut bool,
) -> bool {
    extern "C" {
        fn free(event: *mut XcbGenericEvent) -> ();
    }

    let event = unsafe { (xcb.xcb_poll_for_event)(connection) };
    let event = if event.is_null() {
        return false;
    } else {
        unsafe {
            let r_event = (*event).clone();
            free(event);
            r_event
        }
    };

    let response_type = event.response_type;
    let detail = event.detail as u32;
    let event_xy = (event.event_x, event.event_y);
    let root_xy = (event.root_x as u16, event.root_y as u16); // i16 -> u16

    let text_input = match response_type {
        2 => {
            unsafe {
                let detail = (xkb.xkb_state_key_get_one_sym)(state, detail);

                (xkb.xkb_compose_state_feed)(state2, detail);
            }

            match detail {
                // Enter: Keyboard & NumPad
                36 | 104 => '\n',
                // Left & Right Shift, Alt Gr & NumLock & Esc & Caps Lock
                50 | 62 | 108 | 77 | 9 | 66 => {
                    xkb_state_update_key(xkb, state, detail, true);
                    '\0'
                }
                // Everything else
                _ => {
                    xkb_state_key_get_utf8(xkb, state, state2, detail)
                },
            }
        }
        3 => {
            xkb_state_update_key(xkb, state, detail, false);
            '\0'
        }
        _ => '\0',
    };

    match response_type {
        /*KEY_DOWN*/ 2 => match detail {
            /*ESCAPE*/ 9 => crate::hid::key_press(0, crate::hid::Key::Back),
            /*W*/ 25 => crate::hid::pc::wasd_set(crate::hid::pc::WI),
            /*A*/ 38 => crate::hid::pc::wasd_set(crate::hid::pc::AJ),
            /*S*/ 39 => crate::hid::pc::wasd_set(crate::hid::pc::SK),
            /*D*/ 40 => crate::hid::pc::wasd_set(crate::hid::pc::DL),
            /*I*/ 31 => crate::hid::pc::ijkl_set(crate::hid::pc::WI),
            /*J*/ 44 => crate::hid::pc::ijkl_set(crate::hid::pc::AJ),
            /*K*/ 45 => crate::hid::pc::ijkl_set(crate::hid::pc::SK),
            /*L*/ 46 => crate::hid::pc::ijkl_set(crate::hid::pc::DL),
            _ => {}
        },
        /*KEY_UP*/ 3 => match detail {
            /*ESCAPE*/ 9 => crate::hid::key_release(0, crate::hid::Key::Back),
            /*W*/ 25 => crate::hid::pc::wasd_unset(crate::hid::pc::WI),
            /*A*/ 38 => crate::hid::pc::wasd_unset(crate::hid::pc::AJ),
            /*S*/ 39 => crate::hid::pc::wasd_unset(crate::hid::pc::SK),
            /*D*/ 40 => crate::hid::pc::wasd_unset(crate::hid::pc::DL),
            /*I*/ 31 => crate::hid::pc::ijkl_unset(crate::hid::pc::WI),
            /*J*/ 44 => crate::hid::pc::ijkl_unset(crate::hid::pc::AJ),
            /*K*/ 45 => crate::hid::pc::ijkl_unset(crate::hid::pc::SK),
            /*L*/ 46 => crate::hid::pc::ijkl_unset(crate::hid::pc::DL),
            _ => {}
        },
        /*BUTTON_DOWN*/ 4 => match detail {
            /*Left Click*/ 1 => { // Key::Press
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_press(0, crate::hid::Key::Press);
            }
            /*Middle Click*/ 2 => { // Key::Cmd + Key::Press
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_press(0, crate::hid::Key::L);
                crate::hid::key_press(0, crate::hid::Key::R);
            }
            /*Right Click*/ 3 => { // Key::Menu
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_press(0, crate::hid::Key::Cmd);
                crate::hid::key_press(0, crate::hid::Key::Press);
            }
            /*Scroll Up*/ 4 => { // Left Throttle
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::set_lthrottle(0, 1.0);
            }
            /*Scroll Down*/ 5 => { // Right Throttle
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::set_rthrottle(0, 1.0);
            }
            _ => {} // Ignore all unknown clicks.
        },
        /*BUTTON_UP*/ 5 => match detail {
            /*Left Click*/ 1 => {
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_release(0, crate::hid::Key::Press);
            }
            /*Middle Click*/ 2 => {
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_release(0, crate::hid::Key::L);
                crate::hid::key_release(0, crate::hid::Key::R);
            }
            /*Right Click*/ 3 => {
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::key_release(0, crate::hid::Key::Cmd);
                crate::hid::key_release(0, crate::hid::Key::Press);
            } // queue.right_button_release(*wh, event_xy),
            /*Scroll Up*/ 4 => {
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::set_lthrottle(0, 0.0);
            }
            /*Scroll Down*/ 5 => {
                crate::hid::cursor_coordinates(*wh, event_xy);
                crate::hid::set_rthrottle(0, 0.0);
            }
            _ => {} // Ignore all unknown clicks.
        },
        /*CURSOR_MOVE*/ 6 => {
            crate::hid::cursor_coordinates(*wh, event_xy);
        },
        /*GAIN_FOCUS/RESUME: TODO?*/ 9 => {}
        /*LOSE_FOCUS/PAUSE: TODO?*/ 10 => {}
        /*WINDOW_RESIZE*/ 22 => {
            *wh = root_xy;
            *rtn = true;
        }
        /*WINDOW_SELECT*/ 31 => println!("!SELECT!"),
        /*WINDOW_CLOSE*/ 161 => crate::hid::key_press(0, crate::hid::Key::Back),
        _ => {} // ignore all other messages
    }

    if text_input != '\0' {
        crate::hid::set_char(0, text_input);
    }

    true
}

fn xkb_state_update_key(xkb: &XkbCommonX11, state: *mut c_void, keycode: u32, dn: bool) {
    unsafe {
        (xkb.xkb_state_update_key)(
            state,
            keycode,
            if dn {
                KeyDirection::Down
            } else {
                KeyDirection::Up
            },
        );
    }
}

fn xkb_state_key_get_utf8(xkb: &XkbCommonX11, state: *mut c_void, state2: *mut c_void, key: u32) -> char {
    let status = unsafe { (xkb.xkb_compose_state_get_status)(state2) };

    match status {
        XkbComposeStatus::Composing => {
            return '\0';
        }
        XkbComposeStatus::Composed => {
            let size = unsafe {
                (xkb.xkb_compose_state_get_utf8)(state2, ::std::ptr::null_mut(), 0) as usize + 1
            };
            let mut utf8 = Vec::new();

            utf8.resize(size, b'\0'); // Size + 1 to include NULL byte from XKB.

            let buffer = utf8.as_mut_ptr();

            unsafe {
                (xkb.xkb_compose_state_get_utf8)(state2, buffer, size);
            }

            utf8.pop();

            return ::std::string::String::from_utf8(utf8).unwrap().chars().next().unwrap_or('\0');
        }
        _ => { }
    }

    let size = unsafe {
        (xkb.xkb_state_key_get_utf8)(state, key, ::std::ptr::null_mut(), 0) as usize + 1
    };
    let mut utf8 = Vec::new();

    utf8.resize(size, b'\0'); // Size + 1 to include NULL byte from XKB.

    let buffer = utf8.as_mut_ptr();

    unsafe {
        (xkb.xkb_state_key_get_utf8)(state, key, buffer, size);
    }

    utf8.pop();

    ::std::string::String::from_utf8(utf8).unwrap().chars().next().unwrap_or('\0')
}

/*// Keycode translator
fn key(physical_key: u8) -> Option<u8> {
    Some(match physical_key {
        49 => keyboard::EXT_BACKTICK,
        86 => keyboard::EXT_PLUS,
        63 => keyboard::EXT_ASTERISK,
        61 | 106 => keyboard::SLASH,
        36 | 104 => keyboard::ENTER,
        10 | 87 => keyboard::NUM1,
        11 | 88 => keyboard::NUM2,
        12 | 89 => keyboard::NUM3,
        13 | 83 => keyboard::NUM4,
        14 | 84 => keyboard::NUM5,
        15 | 85 => keyboard::NUM6,
        16 | 79 => keyboard::NUM7,
        17 | 80 => keyboard::NUM8,
        18 | 81 => keyboard::NUM9,
        19 | 90 => keyboard::NUM0,
        60 | 91 => keyboard::PERIOD,
        20 | 82 => keyboard::MINUS,
        21 => keyboard::EQUAL_SIGN,
        22 => keyboard::BACKSPACE,
        23 => keyboard::TAB,
        38 => keyboard::A,
        56 => keyboard::B,
        54 => keyboard::C,
        40 => keyboard::D,
        26 => keyboard::E,
        41 => keyboard::F,
        42 => keyboard::G,
        43 => keyboard::H,
        31 => keyboard::I,
        44 => keyboard::J,
        45 => keyboard::K,
        46 => keyboard::L,
        58 => keyboard::M,
        57 => keyboard::N,
        32 => keyboard::O,
        33 => keyboard::P,
        24 => keyboard::Q,
        27 => keyboard::R,
        39 => keyboard::S,
        28 => keyboard::T,
        30 => keyboard::U,
        55 => keyboard::V,
        25 => keyboard::W,
        53 => keyboard::X,
        29 => keyboard::Y,
        52 => keyboard::Z,
        34 => keyboard::BRACKET_OPEN,
        35 => keyboard::BRACKET_CLOSE,
        37 => keyboard::LCTRL,
        105 => keyboard::RCTRL,
        50 => keyboard::LSHIFT,
        62 => keyboard::RSHIFT,
        64 => keyboard::ALT,
        108 => keyboard::EXT_ALT_GR,
        47 => keyboard::SEMICOLON,
        48 => keyboard::APOSTROPHE,
        51 => keyboard::BACKSLASH,
        59 => keyboard::COMMA,
        65 => keyboard::SPACE,
        77 => keyboard::EXT_NUM_LOCK,
        110 => keyboard::EXT_HOME,
        115 => keyboard::EXT_END,
        112 => keyboard::EXT_PAGE_UP,
        117 => keyboard::EXT_PAGE_DOWN,
        118 => keyboard::EXT_INSERT,
        119 => keyboard::EXT_DELETE,
        111 => keyboard::UP,
        113 => keyboard::LEFT,
        114 => keyboard::RIGHT,
        116 => keyboard::DOWN,
        _ => return None,
    })
}*/
