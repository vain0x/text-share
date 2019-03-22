//! Example actix-web application.
//!
//! This code is adapted from the front page of the [Actix][] website.
//!
//! [actix]: https://actix.rs/docs/

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

mod data;
mod model;

use crate::data::DataInfra;
use crate::model::Model;
use actix_web::{
    http::Method, server, App, AsyncResponder, Form, FromRequest, HttpRequest, HttpResponse,
};
use futures::future::Future;
use std::env;
use std::sync::{Arc, Mutex};

pub(crate) type AsyncState = Arc<Mutex<Model>>;

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
    let state = Arc::clone(req.state());

    let response = get_index(req);

    args.map(move |args| {
        let args = args.into_inner();
        let result = state.lock().unwrap().add(args.key, args.value);

        match result {
            Ok(_) => response,
            Err(()) => HttpResponse::InternalServerError().finish(),
        }
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
    env_logger::init();

    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    // Start a server, configuring the resources to serve.
    server::new(|| {
        let data = DataInfra::new().expect("Couldn't initialize database.");
        let model = Model::new(data);
        let state = Arc::new(Mutex::new(model));

        App::with_state(state)
            .route("/", Method::GET, get_index)
            .route("/", Method::POST, post_index)
            .route("/{key}", Method::GET, get_key)
            .finish()
    })
    .bind(("0.0.0.0", port))
    .map(|x| {
        info!("Listening port {}", port);
        x
    })
    .expect("Can not bind to port 8000")
    .run();
}
