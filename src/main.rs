use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use reqwest::Client;
use rocket::{Data, request::Request, route, Route};
use rocket::tokio::spawn;
use rocket::tokio::time::sleep;
use rocket::serde::Serialize;
use rocket::http::Method::*;
use rocket::shield::Shield;

pub struct RequestTracker {
    requests: Arc<AtomicU64>
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TrackerMessage<'r> {
    name: &'r str,
    rps: u64,
    secret: &'r str
}

pub async fn request_tracker(requests: Arc<AtomicU64>, server_name: &str, control_url: &str, shared_secret: &str) {
    let client = Client::new();

    loop {
        let rps = requests.load(Relaxed);
        requests.store(0, Relaxed);

        let message = TrackerMessage {
            name: server_name,
            rps,
            secret: shared_secret
        };

        spawn(client.post(control_url).json(&message).send());

        let to_wait = Duration::from_millis(1000);
        sleep(to_wait).await;
    }
}

fn handle_request<'r>(req: &'r Request, _: Data<'r>) -> route::BoxFuture<'r> {
    let state = req.rocket().state::<RequestTracker>();

    match state {
        Some(tracker) => {
            tracker.requests.fetch_add(1, Relaxed);
        }
        None => {}
    }

    route::Outcome::from(req, "").pin()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let requests = Arc::new(AtomicU64::new(0));

    let mut routes = vec![];
    for method in &[Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch] {
        routes.push(Route::new(*method, "/<path..>", handle_request));
    }

    let r = rocket::build()
        .manage(RequestTracker { requests: requests.clone() })
        .mount("/", routes)
        .attach(Shield::new()) // disable security headers
        .ignite().await?;

    let figment = r.figment();

    let server_name: String = figment.extract_inner("dstat.server_name").unwrap();
    let shared_secret: String = figment.extract_inner("dstat.shared_secret").unwrap();
    let control_server: String = figment.extract_inner("dstat.control_server").unwrap();

    let control_url = format!("{}/api/v1/reports", control_server);

    spawn({
        async move {
            request_tracker(requests, &server_name, &control_url, &shared_secret).await
        }
    });

    let _ = r.launch().await?;

    Ok(())
}