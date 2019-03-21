//! Example actix-web application.
//!
//! This code is adapted from the front page of the [Actix][] website.
//!
//! [actix]: https://actix.rs/docs/

#[macro_use]
extern crate serde_derive;

use actix_web::{
    http::Method, server, App, AsyncResponder, Form, FromRequest, HttpMessage, HttpRequest,
    HttpResponse,
};
use futures::future::Future;
use std::collections::BTreeMap;
use std::env;
use std::sync::{Arc, Mutex};

struct State {
    map: BTreeMap<String, String>,
}

impl State {
    fn new() -> Self {
        State {
            map: BTreeMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    fn add(&mut self, key: String, value: String) {
        if value.len() >= 10000 {
            return;
        }

        if self.map.len() >= 1000 {
            self.map.clear();
        }

        self.map.insert(key, value);
    }
}

type AsyncState = Arc<Mutex<State>>;

#[derive(Deserialize)]
struct IndexPostArgs {
    key: String,
    value: String,
}

fn get_index(_req: HttpRequest<AsyncState>) -> HttpResponse {
    HttpResponse::Ok().body(include_str!("index.html"))
}

fn post_index(
    req: HttpRequest<AsyncState>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::Error>> {
    let args = Form::<IndexPostArgs>::extract(&req);
    let state = req.state().clone();

    let response = get_index(req);

    args.map(move |args| {
        let args = args.into_inner();
        state.lock().unwrap().add(args.key, args.value);
        response
    })
    .responder()
}

fn get_key(req: HttpRequest<AsyncState>) -> HttpResponse {
    let key = req.match_info().get("key").unwrap_or("");
    let value = match req.state().lock().unwrap().get(key) {
        Some(value) => value.clone(),
        None => return HttpResponse::NotFound().finish(),
    };

    HttpResponse::Ok()
        .header("X-Content-Type-Options", "no-sniff")
        .body(value)
}

fn main() {
    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    // Start a server, configuring the resources to serve.
    server::new(|| {
        App::with_state(Arc::new(Mutex::new(State::new())))
            .route("/", Method::GET, get_index)
            .route("/", Method::POST, post_index)
            .route("/{key}", Method::GET, get_key)
            .finish()
    })
    .bind(("0.0.0.0", port))
    .expect("Can not bind to port 8000")
    .run();
}
