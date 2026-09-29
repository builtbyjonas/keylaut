//! macOS platform backend using native CoreGraphics Event Taps.

use std::fs;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::core::event::{Key, KeyAction, KeyEvent, Modifiers};
use crate::core::state::EngineAction;
use crate::core::KeylautEngine;

const KEYLAUT_MAGIC_USER_DATA: i64 = 0x4B45594C; // "KEYL"

type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CGEventSourceRef = *mut c_void;
type CGEventMask = u64;

const K_CGHID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

const K_CG_EVENT_KEY_DOWN: u32 = 10;
const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFFFFFF;

const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const K_CG_EVENT_SOURCE_USER_DATA: u32 = 42;

const K_CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x00020000;
const K_CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x00040000;
const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;
const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[allow(clippy::duplicated_attributes)]
#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> *mut c_void;

    fn CGEventTapEnable(tap: *mut c_void, enable: bool);

    fn CFMachPortCreateRunLoopSource(
        allocator: *mut c_void,
        port: *mut c_void,
        order: isize,
    ) -> *mut c_void;

    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: *mut c_void);
    static kCFRunLoopCommonModes: *const c_void;

    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);

    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: u64,
        actual_string_length: *mut u64,
        unicode_string: *mut u16,
    );

    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;

    fn CGEventKeyboardSetUnicodeString(
        event: CGEventRef,
        string_length: u64,
        unicode_string: *const u16,
    );

    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(cf: *mut c_void);

    fn AXIsProcessTrusted() -> bool;
}

static ENGINE_HOLDER: Mutex<Option<KeylautEngine>> = Mutex::new(None);
static RUNLOOP_HOLDER: Mutex<Option<usize>> = Mutex::new(None);
static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

/// Checks macOS Accessibility permissions.
pub fn check_permissions() -> Result<(), String> {
    unsafe {
        if AXIsProcessTrusted() {
            Ok(())
        } else {
            Err("Keylaut could not access global keyboard input.\n\n\
                On macOS, enable Keylaut under:\n\
                System Settings → Privacy & Security → Accessibility."
                .to_string())
        }
    }
}

/// Runs the macOS event tap loop.
pub fn run(engine: KeylautEngine) -> Result<(), String> {
    check_permissions()?;

    {
        let mut guard = ENGINE_HOLDER.lock().unwrap();
        *guard = Some(engine);
    }

    let mask: CGEventMask = (1 << K_CG_EVENT_KEY_DOWN) | (1 << K_CG_EVENT_FLAGS_CHANGED);

    unsafe {
        let tap_port = CGEventTapCreate(
            K_CGHID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            mask,
            event_tap_callback,
            std::ptr::null_mut(),
        );

        if tap_port.is_null() {
            return Err(
                "Failed to create macOS CGEventTap. Please grant Accessibility permissions in System Settings."
                    .to_string(),
            );
        }

        let loop_source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap_port, 0);
        if loop_source.is_null() {
            CFRelease(tap_port);
            return Err("Failed to create CFRunLoopSource for Event Tap.".to_string());
        }

        let current_rl = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_rl, loop_source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap_port, true);

        {
            let mut rl_guard = RUNLOOP_HOLDER.lock().unwrap();
            *rl_guard = Some(current_rl as usize);
        }

        log::info!("Keylaut macOS platform backend active.");
        CFRunLoopRun();

        // Cleanup
        CGEventTapEnable(tap_port, false);
        CFRelease(loop_source);
        CFRelease(tap_port);
    }

    Ok(())
}

/// Gracefully stops the running event loop.
pub fn stop() {
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
    let rl = {
        let mut guard = RUNLOOP_HOLDER.lock().unwrap();
        guard.take()
    };
    if let Some(rl_ptr) = rl {
        unsafe {
            CFRunLoopStop(rl_ptr as *mut c_void);
        }
    }
}

unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // If the tap was disabled by timeout, re-enable it
    if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT
        || event_type == K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT
    {
        return event;
    }

    if event.is_null() {
        return event;
    }

    // Ignore synthetic events generated by Keylaut itself
    let user_data = CGEventGetIntegerValueField(event, K_CG_EVENT_SOURCE_USER_DATA);
    if user_data == KEYLAUT_MAGIC_USER_DATA {
        return event;
    }

    if event_type != K_CG_EVENT_KEY_DOWN {
        return event;
    }

    let flags = CGEventGetFlags(event);
    let modifiers = Modifiers {
        shift: (flags & K_CG_EVENT_FLAG_MASK_SHIFT) != 0,
        ctrl: (flags & K_CG_EVENT_FLAG_MASK_CONTROL) != 0,
        alt: (flags & K_CG_EVENT_FLAG_MASK_ALTERNATE) != 0,
        meta: (flags & K_CG_EVENT_FLAG_MASK_COMMAND) != 0,
    };

    let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16;

    let mut actual_len: u64 = 0;
    let mut uni_buf: [u16; 4] = [0; 4];
    CGEventKeyboardGetUnicodeString(event, 4, &mut actual_len, uni_buf.as_mut_ptr());

    let key = match keycode {
        51 => Key::Backspace,
        53 => Key::Escape,
        36 => Key::Enter,
        48 => Key::Tab,
        49 => Key::Space,
        123 => Key::Left,
        124 => Key::Right,
        125 => Key::Down,
        126 => Key::Up,
        115 => Key::Home,
        119 => Key::End,
        116 => Key::PageUp,
        121 => Key::PageDown,
        117 => Key::Delete,
        _ => {
            if actual_len > 0 {
                if let Some(ch) = char::decode_utf16(uni_buf[..actual_len as usize].iter().copied())
                    .next()
                    .and_then(|r| r.ok())
                {
                    Key::Char(ch)
                } else {
                    Key::Other
                }
            } else {
                Key::Other
            }
        }
    };

    let key_event = KeyEvent {
        key,
        modifiers,
        action: KeyAction::Press,
    };

    let action = {
        let mut guard = ENGINE_HOLDER.lock().unwrap();
        if let Some(engine) = guard.as_mut() {
            engine.process_event(key_event)
        } else {
            EngineAction::Pass
        }
    };

    match action {
        EngineAction::Pass => event,
        EngineAction::Suppress => std::ptr::null_mut(),
        EngineAction::Replace { backspaces, text } => {
            // Inject backspaces to delete candidate word
            for _ in 0..backspaces {
                inject_key(51); // 51 is Backspace
            }

            // Inject replacement Unicode characters
            inject_unicode_string(&text);

            // Suppress the trigger event since its effect is already included in `text`
            std::ptr::null_mut()
        }
    }
}

unsafe fn inject_key(keycode: u16) {
    let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, true);
    if !down.is_null() {
        CGEventSetIntegerValueField(down, K_CG_EVENT_SOURCE_USER_DATA, KEYLAUT_MAGIC_USER_DATA);
        CGEventPost(K_CGHID_EVENT_TAP, down);
        CFRelease(down);
    }

    let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, false);
    if !up.is_null() {
        CGEventSetIntegerValueField(up, K_CG_EVENT_SOURCE_USER_DATA, KEYLAUT_MAGIC_USER_DATA);
        CGEventPost(K_CGHID_EVENT_TAP, up);
        CFRelease(up);
    }
}

unsafe fn inject_unicode_string(text: &str) {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    if utf16.is_empty() {
        return;
    }

    let event = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0, true);
    if !event.is_null() {
        CGEventSetIntegerValueField(event, K_CG_EVENT_SOURCE_USER_DATA, KEYLAUT_MAGIC_USER_DATA);
        CGEventKeyboardSetUnicodeString(event, utf16.len() as u64, utf16.as_ptr());
        CGEventPost(K_CGHID_EVENT_TAP, event);
        CFRelease(event);
    }
}

// ---------------------------------------------------------------------------
// macOS Autostart via LaunchAgent
// ---------------------------------------------------------------------------

const LAUNCH_AGENT_LABEL: &str = "com.builtbyjonas.keylaut";

fn launch_agent_path() -> Result<PathBuf, std::io::Error> {
    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LAUNCH_AGENT_LABEL)))
}

pub fn autostart_enable() -> Result<(), std::io::Error> {
    let plist_path = launch_agent_path()?;
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let current_exe = std::env::current_exe()?;
    let exe_str = current_exe.to_string_lossy();

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/dev/null</string>
    <key>StandardOutPath</key>
    <string>/dev/null</string>
</dict>
</plist>
"#,
        label = LAUNCH_AGENT_LABEL,
        exe = exe_str
    );

    fs::write(&plist_path, plist_content)?;

    let _ = Command::new("launchctl")
        .arg("unload")
        .arg(&plist_path)
        .output();

    let status = Command::new("launchctl")
        .arg("load")
        .arg(&plist_path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("launchctl load failed"))
    }
}

pub fn autostart_disable() -> Result<(), std::io::Error> {
    let plist_path = launch_agent_path()?;
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .arg("unload")
            .arg(&plist_path)
            .status();
        let _ = fs::remove_file(&plist_path);
    }
    Ok(())
}

pub fn autostart_status() -> Result<bool, std::io::Error> {
    let plist_path = launch_agent_path()?;
    Ok(plist_path.exists())
}
