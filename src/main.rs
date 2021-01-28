#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate redis;

use uuid::Uuid;
use std::fs::File;
use rocket::{
    http::{Cookies, Cookie},
    request::{self, Request, FromRequest},
    outcome::IntoOutcome,
    request::Form,
    response::Redirect,
};
use redis::Commands;
use dotenv;

mod static_files {
    use super::*;

    #[get("/css/auth.css")]
    pub fn auth_css() -> File {
        File::open("css/auth.css").unwrap()
    }

    #[get("/css/index.css")]
    pub fn index_css() -> File {
        File::open("css/index.css").unwrap()
    }

    #[get("/fonts/CutiveMono-Regular.ttf")]
    pub fn cutive_font() -> File {
        File::open("fonts/CutiveMono-Regular.ttf").unwrap()
    }
}

struct LoggedIn;

impl<'a, 'r> FromRequest<'a, 'r> for LoggedIn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LoggedIn, ()> {
        request.cookies()
            .get_private("api_token")
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|id: String| {
                let client = redis::Client::open("redis://127.0.0.1/").unwrap();
                let mut con = client.get_connection().unwrap();

                match con.get::<_, String>(id) {
                    Ok(_) => Some(LoggedIn{}),
                    Err(_) => None,
                }
            })
            .or_forward(())
    }
}

#[get("/")]
fn index(_l: LoggedIn) -> File {
    File::open("html/index.html").unwrap()
}

#[get("/", rank = 2)]
fn auth() -> File {
    File::open("html/auth.html").unwrap()
}

#[derive(FromForm)]
struct Password{
    password: String
}

#[post("/login", data = "<passwd>")]
fn login(passwd: Form<Password>, mut cookies: Cookies) -> Redirect {
    if passwd.password == "100" {
        let uuid = Uuid::new_v4().as_u128().to_string();

        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();
        let _ : () = con.set_ex(uuid.as_str(), true, 10).unwrap();

        cookies.add_private(Cookie::new("api_token", uuid));
    }
    Redirect::to(uri!(index))
}

#[post("/login", rank = 2)]
fn login_fail() -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login")]
fn login_get() -> Redirect {
    Redirect::to(uri!(index))
}

fn main() {
    dotenv::dotenv().ok();
    rocket::ignite().mount("/", routes![index, auth, login, login_get, login_fail, static_files::auth_css, static_files::index_css, static_files::cutive_font]).launch();
}

