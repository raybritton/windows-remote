use crate::{ARGS, ERROR};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{exit, Stdio};
use std::process::{Command, ExitStatus};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, PostMessageA, WM_CLOSE};

const UPDATE_URL: &str = "https://github.com/raybritton/windows-remote/releases/latest";
const NEW_EXE_FILE_NAME: &str = "nuc_remote.exe.new";
const OLD_EXE_FILE_NAME: &str = "nuc_remote.exe.old";
const EXE_FILE_NAME: &str = "nuc_remote.exe";

pub fn is_old_exe_found() -> bool {
    Path::new(OLD_EXE_FILE_NAME).exists()
}

#[allow(unused_must_use)] //don't care if error occurs
pub fn delete_old_exe_file() {
    fs::remove_file(Path::new(OLD_EXE_FILE_NAME));
}

pub fn kill_foreground() {
    unsafe {
        let window = GetForegroundWindow();
        PostMessageA(window, WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}

pub fn kill_non_responsive() {
    if let Err(err) = kill_non_responsive_impl() {
        let mut guard = ERROR.lock().unwrap();
        *guard = format!("Kill frozen error<br>{}", err);
    } else {
        let mut guard = ERROR.lock().unwrap();
        *guard = String::new();
    }
}

fn kill_non_responsive_impl() -> std::io::Result<ExitStatus> {
    Command::new("C:\\Windows\\System32\\taskkill.exe")
        .args(vec!["/f", "/fi", "\"status eq not responding\""])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()
}

pub fn suicide() -> bool {
    if let Err(err) = kill_self_impl() {
        let mut guard = ERROR.lock().unwrap();
        *guard = format!("Killing self:<br>{}", err);
        false
    } else {
        let mut guard = ERROR.lock().unwrap();
        *guard = String::new();
        true
    }
}

pub fn kill_self_impl() -> std::io::Result<ExitStatus> {
    Command::new("C:\\Windows\\System32\\taskkill.exe")
        .args(vec!["/f", "/im", "nuc_remote.exe"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()
}

pub fn update_self() {
    if let Err(err) = update_self_impl() {
        let mut guard = ERROR.lock().unwrap();
        *guard = format!("Updating error: <br>{}", err);
    } else {
        let mut guard = ERROR.lock().unwrap();
        *guard = String::new();
    }
}

fn get_latest_url() -> Result<String, String> {
    let resp = ureq::builder()
        .redirects(0)
        .build()
        .get(UPDATE_URL)
        .call()
        .map_err(|e| format!("get latest url:<br> {}", e))?;

    if resp.status() == 302 {
        if let Some(new_url) = resp.header("location") {
            let exe_url = format!("{}/nuc_remote.exe", new_url).replace("tag", "download");
            Ok(exe_url)
        } else {
            Err(String::from(
                "Calling latest url gave redirect response but no new url",
            ))
        }
    } else {
        Err(format!(
            "Calling latest url did not result with redirect, was {}",
            resp.status()
        ))
    }
}

fn get_exe(url: String) -> Result<Vec<u8>, String> {
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("get exe:<br> {}", e))?;
    if resp.status() == 200 {
        let bytes: Result<Vec<u8>, _> = resp.into_reader().bytes().collect();
        match bytes {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(format!("Reading exe bytes:<br>{}", e)),
        }
    } else {
        Err(format!("Calling exe url failed with {}", resp.status()))
    }
}

fn write_exe_file(bytes: Vec<u8>) -> Result<(), String> {
    fs::write(NEW_EXE_FILE_NAME, bytes).map_err(|e| format!("Unable to write exe file: {}", e))?;
    Ok(())
}

fn rename_exe_files() -> Result<(), String> {
    fs::rename(EXE_FILE_NAME, OLD_EXE_FILE_NAME)
        .map_err(|e| format!("renaming current exe:<br>{}", e))?;
    fs::rename(NEW_EXE_FILE_NAME, EXE_FILE_NAME)
        .map_err(|e| format!("renaming new exe:<br>{}", e))?;
    Ok(())
}

pub fn update_self_impl() -> Result<(), String> {
    let latest_url = get_latest_url()?;
    let exe_bytes = get_exe(latest_url)?;
    write_exe_file(exe_bytes)?;
    rename_exe_files()?;
    Ok(())
}

pub fn reboot() {
    let args = ARGS.lock().unwrap();
    let mut iter = args.iter();
    Command::new(iter.next().unwrap())
        .args(iter)
        .spawn()
        .unwrap();
    exit(0);
}
