use crate::input::{create_keyboard_device, emit_key};
use crate::state::SharedState;
use evdev::{
    AbsInfo, AbsoluteAxisType, EventType, InputEvent, Key, RelativeAxisType, UinputAbsSetup,
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder,
};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

const ABS_MAX: i32 = 65535;

struct OutputLogical {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

struct ScreenMetrics {
    min_x: i32,
    min_y: i32,
    virtual_w: i32,
    virtual_h: i32,
    outputs: HashMap<String, OutputLogical>,
}

pub fn run(state_arc: &SharedState) -> Result<(), String> {
    if !is_niri() {
        return Err("Niri mode requires Niri environment.".to_string());
    }

    let mut kb_device = create_keyboard_device()?;
    let mut pointer = create_pointer_device()?;

    thread::sleep(Duration::from_millis(500));

    loop {
        let s = { state_arc.lock().unwrap().clone() };
        if !s.running || s.mode != 2 {
            break;
        }

        // Under Niri, we cannot easily check mouse cursor position globally,
        // but we can check if the focused window is not Roblox.
        if s.user_safe && is_user_active() {
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        let metrics = match get_screen_metrics() {
            Some(m) => m,
            None => {
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        let workspace_outputs = match get_workspace_outputs() {
            Some(w) => w,
            None => {
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        // Get window list
        let windows_output = Command::new("niri")
            .args(["msg", "--json", "windows"])
            .output()
            .map_err(|e| e.to_string())?;
        let windows_json: Value =
            serde_json::from_slice(&windows_output.stdout).unwrap_or(Value::Null);

        let mut target_windows = Vec::new();
        let my_pid = i64::from(std::process::id());

        if let Some(arr) = windows_json.as_array() {
            for win in arr {
                let app_id = win.get("app_id").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let title = win.get("title").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let pid = win.get("pid").and_then(|v| v.as_i64()).unwrap_or(0);
                let id = win.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let workspace_id = win.get("workspace_id").and_then(|v| v.as_i64());
                let is_floating = win.get("is_floating").and_then(|v| v.as_bool()).unwrap_or(false);

                let layout = win.get("layout");
                let win_size = layout.and_then(|l| l.get("window_size")).and_then(|s| s.as_array());
                let tile_pos = layout.and_then(|l| l.get("tile_pos_in_workspace_view")).and_then(|p| p.as_array());

                if pid == my_pid {
                    continue;
                }

                if app_id.contains("sober") || title.contains("roblox") || app_id.contains("vinegar") || app_id == "sober" {
                    let win_w = win_size.and_then(|s| s[0].as_i64()).unwrap_or(1920) as i32;
                    let win_h = win_size.and_then(|s| s[1].as_i64()).unwrap_or(1080) as i32;
                    
                    target_windows.push((id, win_w, win_h, workspace_id, is_floating, tile_pos.cloned()));
                }
            }
        }

        if target_windows.is_empty() {
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        if !s.multi_instance {
            target_windows.truncate(1);
        }

        {
            state_arc.lock().unwrap().action_active = true;
        }

        let initial_window = get_active_window_id();

        for (id, win_w, win_h, workspace_id, is_floating, tile_pos) in target_windows {
            // Find the output this workspace is on
            let ws_id = match workspace_id {
                Some(w) => w,
                None => continue,
            };

            let out_name = match workspace_outputs.get(&ws_id) {
                Some(n) => n,
                None => continue,
            };

            let out_logical = match metrics.outputs.get(out_name) {
                Some(o) => o,
                None => continue,
            };

            // Calculate center and top-left of the window
            let (win_x, win_y) = if is_floating {
                if let Some(ref tp) = tile_pos {
                    let fx = tp[0].as_f64().unwrap_or(0.0) as i32;
                    let fy = tp[1].as_f64().unwrap_or(0.0) as i32;
                    (out_logical.x + fx, out_logical.y + fy)
                } else {
                    (
                        out_logical.x + (out_logical.w - win_w) / 2,
                        out_logical.y + (out_logical.h - win_h) / 2,
                    )
                }
            } else {
                (
                    out_logical.x + (out_logical.w - win_w) / 2,
                    out_logical.y + (out_logical.h - win_h) / 2,
                )
            };

            let cx = win_x + win_w / 2;
            let cy = win_y + win_h / 2;

            if s.hides_game {
                // In Niri, to show/hide game, we can move it from/to its workspace
                let _ = Command::new("niri")
                    .args([
                        "msg",
                        "action",
                        "move-window-to-workspace",
                        "--window-id",
                        &id.to_string(),
                        "--focus",
                        "true",
                        &ws_id.to_string(),
                    ])
                    .output();
                thread::sleep(Duration::from_millis(200));
            }

            // Focus window
            let _ = Command::new("niri")
                .args(["msg", "action", "focus-window", "--id", &id.to_string()])
                .output();
            thread::sleep(Duration::from_millis(150));

            // Move cursor to center of the window
            // Subtract metrics.min_x/y to convert to virtual screen space relative to top-left of entire layout
            let virtual_x = cx - metrics.min_x;
            let virtual_y = cy - metrics.min_y;
            warp_cursor(&mut pointer, virtual_x, virtual_y, metrics.virtual_w, metrics.virtual_h);
            thread::sleep(Duration::from_millis(150));

            // Left Click to ensure focus/interact
            let _ = pointer.emit(&[
                InputEvent::new(EventType::KEY, Key::BTN_LEFT.0, 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
            thread::sleep(Duration::from_millis(30));
            let _ = pointer.emit(&[
                InputEvent::new(EventType::KEY, Key::BTN_LEFT.0, 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
            thread::sleep(Duration::from_millis(50));

            // Auto Reconnect Check
            if s.auto_reconnect {
                let check_x = win_x + (win_w - 400) / 2 + 10;
                let check_y = win_y + (win_h - 250) / 2 + 10;
                if let Some((r, g, b)) = get_pixel_color(check_x as i64, check_y as i64) {
                    if r == 57 && g == 59 && b == 61 {
                        let target_x = win_x + (win_w - 400) / 2 + (400 - 161 - 27) + (161 / 2);
                        let target_y = win_y + (win_h - 250) / 2 + (250 - 34 - 21) + (34 / 2);
                        
                        let click_vx = target_x - metrics.min_x;
                        let click_vy = target_y - metrics.min_y;

                        warp_cursor(&mut pointer, click_vx, click_vy, metrics.virtual_w, metrics.virtual_h);
                        thread::sleep(Duration::from_millis(100));

                        for _ in 0..3 {
                            let _ = pointer.emit(&[
                                InputEvent::new(EventType::KEY, Key::BTN_LEFT.0, 1),
                                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                            ]);
                            thread::sleep(Duration::from_millis(30));
                            let _ = pointer.emit(&[
                                InputEvent::new(EventType::KEY, Key::BTN_LEFT.0, 0),
                                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                            ]);
                            thread::sleep(Duration::from_millis(30));
                        }

                        warp_cursor(&mut pointer, virtual_x, virtual_y, metrics.virtual_w, metrics.virtual_h);
                    }
                }
            }

            // Keyboard actions
            if s.jump {
                let _ = emit_key(&mut kb_device, Key::KEY_SPACE, true);
                thread::sleep(Duration::from_millis(30));
                let _ = emit_key(&mut kb_device, Key::KEY_SPACE, false);
            }
            if s.walk {
                let _ = emit_key(&mut kb_device, Key::KEY_W, true);
                thread::sleep(Duration::from_millis(150));
                let _ = emit_key(&mut kb_device, Key::KEY_W, false);
            }
            if s.spin_jiggle {
                let _ = emit_key(&mut kb_device, Key::KEY_I, true);
                thread::sleep(Duration::from_millis(30));
                let _ = emit_key(&mut kb_device, Key::KEY_I, false);
                thread::sleep(Duration::from_millis(50));
                let _ = emit_key(&mut kb_device, Key::KEY_O, true);
                thread::sleep(Duration::from_millis(30));
                let _ = emit_key(&mut kb_device, Key::KEY_O, false);
            }

            if s.hides_game {
                // Move window to special workspace to hide it
                let _ = Command::new("niri")
                    .args([
                        "msg",
                        "action",
                        "move-window-to-workspace",
                        "--window-id",
                        &id.to_string(),
                        "--focus",
                        "false",
                        "special",
                    ])
                    .output();
            }
        }

        // Restore initial focused window if needed
        if let Some(win_id) = initial_window {
            let _ = Command::new("niri")
                .args(["msg", "action", "focus-window", "--id", &win_id.to_string()])
                .output();
            thread::sleep(Duration::from_millis(100));
        }

        {
            state_arc.lock().unwrap().action_active = false;
        }

        if responsive_sleep(state_arc, 2) {
            break;
        }
    }

    Ok(())
}

fn is_niri() -> bool {
    crate::state::AppState::is_niri()
}

fn get_active_window_id() -> Option<i64> {
    let output = Command::new("niri").args(["msg", "--json", "windows"]).output().ok()?;
    let json: Value = serde_json::from_slice(&output.stdout).ok()?;
    if let Some(arr) = json.as_array() {
        for win in arr {
            if win.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false) {
                return win.get("id").and_then(|v| v.as_i64());
            }
        }
    }
    None
}

fn is_user_active() -> bool {
    // If the focused window is not Sober/Roblox, that's one indicator.
    // However, since we cannot easily get cursor coordinates in Niri without root,
    // we return false to keep executing, which is a safe fallback.
    false
}

fn get_pixel_color(x: i64, y: i64) -> Option<(u8, u8, u8)> {
    let output = Command::new("grim")
        .args(["-t", "ppm", "-g", &format!("{x},{y} 1x1"), "-"])
        .output()
        .ok()?;
    let bytes = output.stdout;
    if bytes.len() >= 13 && bytes.starts_with(b"P6") {
        let l = bytes.len();
        return Some((bytes[l - 3], bytes[l - 2], bytes[l - 1]));
    }
    None
}

fn create_pointer_device() -> Result<VirtualDevice, String> {
    let abs_x = UinputAbsSetup::new(
        AbsoluteAxisType::ABS_X,
        AbsInfo::new(0, 0, ABS_MAX, 0, 0, 0),
    );
    let abs_y = UinputAbsSetup::new(
        AbsoluteAxisType::ABS_Y,
        AbsInfo::new(0, 0, ABS_MAX, 0, 0, 0),
    );

    let mut keys = evdev::AttributeSet::<Key>::new();
    keys.insert(Key::BTN_LEFT);

    let mut rel = evdev::AttributeSet::<RelativeAxisType>::new();
    rel.insert(RelativeAxisType::REL_X);
    rel.insert(RelativeAxisType::REL_Y);

    VirtualDeviceBuilder::new()
        .map_err(|e: std::io::Error| e.to_string())?
        .name("AntiAFK Virtual Pointer")
        .with_keys(&keys)
        .map_err(|e: std::io::Error| e.to_string())?
        .with_relative_axes(&rel)
        .map_err(|e: std::io::Error| e.to_string())?
        .with_absolute_axis(&abs_x)
        .map_err(|e: std::io::Error| e.to_string())?
        .with_absolute_axis(&abs_y)
        .map_err(|e: std::io::Error| e.to_string())?
        .build()
        .map_err(|e: std::io::Error| {
            format!("Pointer device creation failed: {e}. Run: sudo chmod 666 /dev/uinput")
        })
}

fn warp_cursor(device: &mut VirtualDevice, x: i32, y: i32, screen_w: i32, screen_h: i32) {
    if screen_w <= 0 || screen_h <= 0 {
        return;
    }
    let abs_x = (x as i64 * ABS_MAX as i64 / screen_w as i64).clamp(0, ABS_MAX as i64) as i32;
    let abs_y = (y as i64 * ABS_MAX as i64 / screen_h as i64).clamp(0, ABS_MAX as i64) as i32;

    let _ = device.emit(&[
        InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, abs_x),
        InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, abs_y),
        InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
    ]);
}

fn get_screen_metrics() -> Option<ScreenMetrics> {
    let output = Command::new("niri").args(["msg", "--json", "outputs"]).output().ok()?;
    let json: Value = serde_json::from_slice(&output.stdout).ok()?;

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut outputs = HashMap::new();

    if let Some(obj) = json.as_object() {
        for (name, out_val) in obj {
            if let Some(logical) = out_val.get("logical") {
                let x = logical.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = logical.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let w = logical.get("width").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                let h = logical.get("height").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x + w > max_x {
                    max_x = x + w;
                }
                if y + h > max_y {
                    max_y = y + h;
                }

                outputs.insert(name.clone(), OutputLogical { x, y, w: w, h: h });
            }
        }
    }

    if min_x == i32::MAX || min_y == i32::MAX || max_x == i32::MIN || max_y == i32::MIN {
        return None;
    }

    Some(ScreenMetrics {
        min_x,
        min_y,
        virtual_w: max_x - min_x,
        virtual_h: max_y - min_y,
        outputs,
    })
}

fn get_workspace_outputs() -> Option<HashMap<i64, String>> {
    let output = Command::new("niri").args(["msg", "--json", "workspaces"]).output().ok()?;
    let json: Value = serde_json::from_slice(&output.stdout).ok()?;
    let mut map = HashMap::new();
    if let Some(arr) = json.as_array() {
        for ws in arr {
            let id = ws.get("id").and_then(|v| v.as_i64());
            let out_name = ws.get("output").and_then(|v| v.as_str());
            if let (Some(i), Some(o)) = (id, out_name) {
                map.insert(i, o.to_string());
            }
        }
    }
    Some(map)
}

fn responsive_sleep(state_arc: &SharedState, mode: usize) -> bool {
    let interval = state_arc.lock().unwrap().interval_seq;
    for _ in 0..interval {
        thread::sleep(Duration::from_secs(1));
        let s = state_arc.lock().unwrap().clone();
        if !s.running || s.mode != mode {
            return true;
        }
    }
    false
}
