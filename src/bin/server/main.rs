#![feature(decl_macro)]

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

fn main() {
    let route = routes![user_id];
    rocket::ignite().mount("/", route).launch();
}
