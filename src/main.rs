#![allow(unused, unreachable_code)]
use std::collections::HashMap;

use async_stream::stream;
use axum::{
    Router, debug_handler,
    extract::Query,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive},
    },
    routing::get,
};
use datastar::{
    Sse,
    axum::ReadSignals,
    consts::FragmentMergeMode,
    prelude::{ExecuteScript, MergeFragments, MergeSignals, RemoveSignals},
};
use futures::stream;
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use tower_http::services::ServeDir;

// TODO:
// - [x] Lets try the JS execution
// - [x] wth is signals o
//
// what to they mean by this?
// Signals beginning with an underscore are considered local signals and are not included in
// requests to the --> backend by default <-- You can include them by setting the includeLocal option to
// true.
// https://data-star.dev/reference/attribute_plugins#data-signals

macro_rules! sse_stream {
    ($($e:expr),*) => {
        Sse(stream!(
        $(
            yield $e;
        )*
        ))
    };
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(mainpage))
        .route("/test", get(looper));

    const IP_PORT: &str = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(IP_PORT).await.unwrap();

    println!("Listening on http://{IP_PORT}");

    axum::serve(listener, app).await.unwrap();
}

async fn looper() -> impl IntoResponse {
    Sse(stream! {
        for i in 0..10 {
            yield MergeFragments::new(html!(p {(i)})).merge_mode(FragmentMergeMode::Append).selector("#target");
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

    })
}

async fn mainpage() -> Markup {
    html!(
        (DOCTYPE)
        head {
        title {"Hello world!"}
            script type="module" src="/static/datastar.js" {}
            script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
        }

        body {
            button data-on-click="@get('/test')" {"Press me"}
            div {
                input data-bind-myvar placeholder="write shit.." {}
                div data-text="$myvar" {}
            }
            div class="flex flex-row gap-4" id="target" {}
            }
    )
}
