use crate::ERROR;
use std::process::ExitStatus;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GetForegroundWindow, PostMessageA, WM_CLOSE,
};

pub fn kill_foreground() {
    unsafe {
        let window = GetForegroundWindow();
        PostMessageA(window, WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}

pub fn kill_non_responsive() {
    if let Err(err) = kill_non_responsive_impl() {
        let mut guard = ERROR.lock().unwrap();
        *guard = format!("Kill frozen error\n{}", err.to_string());
    } else {
        let mut guard = ERROR.lock().unwrap();
        *guard = String::new();
    }
}

fn kill_non_responsive_impl() -> std::io::Result<ExitStatus> {
    std::process::Command::new(
        "C:\\Windows\\System32\\taskkill.exe /f /fi \"status eq not responding\"",
    )
    .spawn()?
    .wait()
}
