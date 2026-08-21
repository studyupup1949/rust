use crate::state::{SharedState, APP_ID};
use notify_rust::Notification;
use serde_json::Value;
use std::collections::HashSet;
use std::process::Command;
use std::thread;
use std::time::Duration;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub fn start_backend(state: SharedState) {
    let state_auto = state.clone();
    thread::spawn(move || {
        let mut sober_absent_ticks = 0;
        loop {
            let (auto_start, is_running, manually_stopped) = {
                let s = state_auto.lock().unwrap();
                (s.auto_start, s.running, s.manually_stopped)
            };

            if check_sober_running() {
                sober_absent_ticks = 0;
                if auto_start && !is_running && !manually_stopped {
                    state_auto.lock().unwrap().running = true;
                    notify("Sober detected. Anti-AFK enabled.");
                }
            } else {
                sober_absent_ticks += 1;
                if sober_absent_ticks >= 2 {
                    if is_running {
                        let mut s = state_auto.lock().unwrap();
                        s.running = false;
                        s.manually_stopped = false;
                        notify("Sober closed. Anti-AFK disabled.");
                    } else if manually_stopped {
                        state_auto.lock().unwrap().manually_stopped = false;
                    }
                }
            }
            thread::sleep(Duration::from_secs(7));
        }
    });

    let state_fps = state.clone();
    thread::spawn(move || {
        let mut throttled_pids: HashSet<i32> = HashSet::new();
        let my_pid = std::process::id().to_string();
        loop {
            let (enabled, fps_limit, stop_on_focus, is_running, action_active) = {
                let s = state_fps.lock().unwrap();
                (s.fps_capper, s.fps_limit, s.stop_limit_on_focus, s.running, s.action_active)
            };

            let should_throttle = enabled && is_running && fps_limit > 0 && !action_active && !(stop_on_focus && is_focused_sober());


            if should_throttle {
                let main_pids = get_all_sober_pids(&my_pid);
                let pids: Vec<i32> = main_pids.iter().filter_map(|p| p.parse::<i32>().ok()).collect();
                
                for &pid in &pids {
                    throttled_pids.insert(pid);
                }

                if !pids.is_empty() {
                    let limit = fps_limit.clamp(3, 95) as u64;
                    let run_time = limit;
                    let sleep_time = 100 - limit;

                    for &pid in &pids {
                        let res = kill(Pid::from_raw(pid), Signal::SIGCONT);
                        if let Err(e) = res {
                            eprintln!("Failed to SIGCONT {}: {:?}", pid, e);
                        }
                    }
                    thread::sleep(Duration::from_millis(run_time));

                    for &pid in &pids {
                        let res = kill(Pid::from_raw(pid), Signal::SIGSTOP);
                        if let Err(e) = res {
                            eprintln!("Failed to SIGSTOP {}: {:?}", pid, e);
                        }
                    }
                    thread::sleep(Duration::from_millis(sleep_time));
                } else {
                    thread::sleep(Duration::from_millis(200));
                }
            } else {
                if !throttled_pids.is_empty() {
                    for pid in &throttled_pids {
                        let _ = kill(Pid::from_raw(*pid), Signal::SIGCONT);
                    }
                    throttled_pids.clear();
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    });

    let state_main = state.clone();
    thread::spawn(move || {
        loop {
            let (is_running, mode) = {
                let s = state_main.lock().unwrap();
                (s.running, s.mode)
            };

            if is_running {
                let res = match mode {
                    0 => crate::inputs::swapper::run(&state_main),
                    1 => crate::inputs::plasma::run(&state_main),
                    2 => crate::inputs::niri::run(&state_main),
                    _ => Ok(()),
                };

                if let Err(e) = res {
                    let mut s = state_main.lock().unwrap();
                    s.running = false;
                    notify(&format!("Error: {e}"));
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn notify(msg: &str) {
    let _ = Notification::new()
        .appname(APP_ID)
        .summary("AntiAFK-RBX")
        .body(msg)
        .icon(APP_ID)
        .timeout(5000)
        .show();
}

fn is_focused_sober() -> bool {
    if is_hyprland() {
        if let Ok(out) = Command::new("hyprctl").args(["activewindow", "-j"]).output() {
            if let Ok(json) = serde_json::from_slice::<Value>(&out.stdout) {
                let class = json.get("class").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                return class.contains("sober") || class.contains("roblox") || class.contains("vinegar");
            }
        }
    } else if is_plasma() {
        let qdbus = if Command::new("qdbus6").arg("--version").status().is_ok() { "qdbus6" } else { "qdbus" };
        let script = r#"
            var w = workspace.activeWindow;
            if (w) {
                var cls = (w.resourceClass || "").toLowerCase();
                var title = (w.caption || "").toLowerCase();
                var app = (w.desktopFileName || "").toLowerCase();
                var isSober = (cls.indexOf("sober") !== -1 || cls.indexOf("roblox") !== -1 || 
                               cls.indexOf("vinegar") !== -1 || app.indexOf("sober") !== -1) && 
                               title.indexOf("antiafk") === -1;
                print("ANTIAFK_FOCUS_STATE:" + isSober);
            } else {
                print("ANTIAFK_FOCUS_STATE:false");
            }
        "#;
        
        let script_path = "/tmp/antiafk_backend_focus.js";
        let _ = std::fs::write(script_path, script);
        
        let _ = Command::new(qdbus).args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.unloadScript", "antiafk_backend_focus"]).output();
        
        if let Ok(out) = Command::new(qdbus).args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.loadScript", script_path, "antiafk_backend_focus"]).output() {
            let id_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(id) = id_str.parse::<i32>() {
                let script_obj = format!("/Scripting/Script{}", id);
                let _ = Command::new(qdbus).args(["org.kde.KWin", &script_obj, "org.kde.kwin.Script.run"]).output();
                
                thread::sleep(Duration::from_millis(200));
                
                if let Ok(output) = Command::new("journalctl")
                    .args(["--user", "-n", "30", "--since", "10 seconds ago", "--no-pager", "-o", "cat"])
                    .output()
                {
                    let text = String::from_utf8_lossy(&output.stdout);
                    for line in text.lines().rev() {
                        if let Some(pos) = line.find("ANTIAFK_FOCUS_STATE:") {
                            return line[pos + 20..].trim() == "true";
                        }
                    }
                }
            }
        }
    } else if is_niri() {
        if let Ok(out) = Command::new("niri").args(["msg", "--json", "windows"]).output() {
            if let Ok(json) = serde_json::from_slice::<Value>(&out.stdout) {
                if let Some(arr) = json.as_array() {
                    for win in arr {
                        if win.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let app_id = win.get("app_id").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                            let title = win.get("title").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                            return app_id.contains("sober") || title.contains("roblox") || app_id.contains("vinegar") || app_id == "sober";
                        }
                    }
                }
            }
        }
    }
    false
}



fn check_sober_running() -> bool {
    !get_all_sober_pids(&std::process::id().to_string()).is_empty()
}



fn get_all_sober_pids(exclude_pid: &str) -> Vec<String> {
    let mut target_pids = HashSet::new();
    let my_pid = exclude_pid.to_string();

    let output = Command::new("pgrep").args(["-if", "sober|roblox|vinegar|Sober.bin"]).output().ok();
    if let Some(out) = output {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            let pid = line.trim().to_string();
            if !pid.is_empty() && pid != my_pid {
                if let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) {
                    let comm_lower = comm.to_lowercase();
                    if comm_lower.contains("antiafk") {
                        continue;
                    }
                }

                target_pids.insert(pid.clone());

                if let Ok(cgroup) = std::fs::read_to_string(format!("/proc/{pid}/cgroup")) {
                    for cg_line in cgroup.lines() {
                        if let Some(path) = cg_line.split("::").nth(1) {
                            if !path.is_empty() && path != "/" {
                                let path_lower = path.to_lowercase();
                                if (path_lower.contains("sober") || path_lower.contains("vinegar") || path_lower.contains("roblox") || path_lower.contains("flatpak"))
                                    && !path_lower.contains("inir")
                                    && !path_lower.contains("antiafk")
                                    && !path_lower.contains("kitty")
                                    && !path_lower.contains("terminal")
                                {
                                    let cgroup_dir = format!("/sys/fs/cgroup{}", path);
                                    if let Ok(procs) = std::fs::read_to_string(format!("{}/cgroup.procs", cgroup_dir)) {
                                        for proc_line in procs.lines() {
                                            let p = proc_line.trim().to_string();
                                            if !p.is_empty() && p != my_pid {
                                                target_pids.insert(p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    target_pids.into_iter().collect()
}

pub fn is_hyprland() -> bool {
    crate::state::AppState::is_hyprland()
}
pub fn is_plasma() -> bool {
    crate::state::AppState::is_plasma()
}
pub fn is_niri() -> bool {
    crate::state::AppState::is_niri()
}
