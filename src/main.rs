mod methods;

use crate::methods::{kill_foreground, shutdown};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use std::env::args;
use std::io;
use std::sync::{Arc, Mutex};

const HTML_MENU: &str = include_str!("menu.html");
const HTML_FUNCTION: &str = include_str!("function.html");

lazy_static! {
    pub static ref ERROR: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref TITLE: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("??? Menu")));
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let addr = match args().nth(1) {
        None => {
            eprintln!("No address passed");
            return Ok(());
        }
        Some(addr) => addr,
    };
    if let Some(text) = args().nth(2) {
        let mut guard = TITLE.lock().unwrap();
        *guard = text.clone();
    }
    println!("Starting server on {addr}");
    HttpServer::new(|| {
        App::new()
            .service(menu)
            .service(kill_frozen)
            .service(kill_foreground_program)
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

    let html = HTML_MENU
        .replace("[[error_text]]", error_text)
        .replace("[[title]]", title_text);
    HttpResponse::Ok().body(html)
}

#[get("/kill_foreground")]
async fn kill_foreground_program() -> impl Responder {
    kill_foreground();
    HttpResponse::Ok().body(HTML_FUNCTION)
}

#[get("/kill_frozen")]
async fn kill_frozen() -> impl Responder {
    kill_frozen();
    HttpResponse::Ok().body(HTML_FUNCTION)
}
