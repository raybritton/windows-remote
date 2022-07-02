#![windows_subsystem = "windows"]
mod methods;

use crate::methods::{
    delete_old_exe_file, is_old_exe_found, kill_foreground, kill_non_responsive, reboot, suicide,
    update_self,
};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use chrono::{Local, Timelike};
use lazy_static::lazy_static;
use std::env::args;
use std::fs::File;
use std::io;
use std::io::LineWriter;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;
use uuid::Uuid;

const HTML_MENU: &str = include_str!("menu.html");
const HTML_FUNCTION: &str = include_str!("function.html");

lazy_static! {
    pub static ref ERROR: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref TITLE: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("??? Menu")));
    pub static ref ARGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref ID: Uuid = Uuid::new_v4();
    pub static ref LOG_DIR: String = format!("C:/Users/Ray_B/Documents/logs/{}", *ID);
    pub static ref LOGGER: Arc<Mutex<LineWriter<File>>> = Arc::new(Mutex::new(make_logger()));
}

fn make_logger() -> LineWriter<File> {
    LineWriter::new(File::create(Path::new(LOG_DIR.as_str())).unwrap())
}

pub fn log(text: &str) {
    LOGGER
        .lock()
        .unwrap()
        .write_all(format!("[{}] [{}]: {}\n", *ID, time(), text).as_bytes())
        .unwrap();
}

fn time() -> String {
    let now = Local::now();
    format!(
        "{:0>2}:{:0>2}:{:0>2}",
        now.hour(),
        now.minute(),
        now.second()
    )
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    //wait for current server (if alive) to exit
    sleep(Duration::from_secs(1));
    delete_old_exe_file();
    let mut args_guard = ARGS.lock().unwrap();
    *args_guard = args().collect();
    drop(args_guard);
    let addr = match args().nth(1) {
        None => {
            eprintln!("No address passed");
            return Ok(());
        }
        Some(addr) => addr,
    };
    if let Some(text) = args().nth(2) {
        let mut guard = TITLE.lock().unwrap();
        *guard = text;
    }
    println!("Starting server on {addr}");
    HttpServer::new(|| {
        App::new()
            .service(menu)
            .service(kill_frozen)
            .service(kill_foreground_program)
            .service(update)
            .service(kill_self)
            .service(restart)
    })
    .bind((addr, 45874))?
    .run()
    .await
}

#[get("/")]
async fn menu() -> impl Responder {
    let guard = ERROR.lock().unwrap();
    let error_text = guard.as_ref();
    let guard = TITLE.lock().unwrap();
    let title_text = guard.as_ref();

    let update_class = if is_old_exe_found() { "" } else { "hidden" };

    let html = HTML_MENU
        .replace("[[error_text]]", error_text)
        .replace("[[title]]", title_text)
        .replace("[[update_class]]", update_class);

    HttpResponse::Ok().body(html)
}

#[get("/kill_foreground")]
async fn kill_foreground_program() -> impl Responder {
    kill_foreground();
    HttpResponse::Ok().body(HTML_FUNCTION)
}

#[get("/kill_frozen")]
async fn kill_frozen() -> impl Responder {
    kill_non_responsive();
    HttpResponse::Ok().body(HTML_FUNCTION)
}

#[get("/kill_self")]
async fn kill_self() -> impl Responder {
    spawn(|| {
        sleep(Duration::from_millis(500));
        suicide();
    });
    HttpResponse::Ok().body(HTML_FUNCTION)
}

#[get("/update")]
async fn update() -> impl Responder {
    spawn(|| {
        sleep(Duration::from_millis(500));
        update_self();
    });
    HttpResponse::Ok().body(HTML_FUNCTION)
}

#[get("/restart")]
async fn restart() -> impl Responder {
    spawn(|| {
        sleep(Duration::from_millis(500));
        reboot();
    });
    HttpResponse::Ok().body(HTML_FUNCTION)
}
