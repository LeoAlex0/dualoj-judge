#![feature(decl_macro)]

use std::env;

use rocket::http::Cookies;

#[macro_use]
extern crate rocket;

#[get("/uid")]
fn user_id(mut cookies: Cookies) -> String {
    match cookies.get_private("uid") {
        Some(cookie) => format!("User ID:{}", cookie.value()),
        None => format!("Non Login"),
    }
}

#[get("/env/<env>")]
fn get_env(env: String) -> String {
    env::var(env).unwrap_or("<NULL>".into())
}

#[get("/")]
fn default() -> String {
    "welcome to judger, try: /env or /uid".into()
}

fn main() {
    let route = routes![user_id, get_env];
    rocket::ignite().mount("/", route).launch();
}
