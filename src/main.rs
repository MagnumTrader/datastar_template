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
    const MSG: &'static str = "Hello World!";
    Sse(stream! {
        for i in 0..MSG.len() {
            yield MergeFragments::new(html!(p class="text-3xl" {(MSG[..i])})).merge_mode(FragmentMergeMode::Inner).selector("#target").into_event();
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
        yield MergeFragments::new(html!(p class="text-3xl" {({ 
            let mut s = MSG.to_string();
            s.push_str("👋"); 
            s
        })}
        )).merge_mode(FragmentMergeMode::Inner).selector("#target").into_event();
        yield MergeSignals::new("{sent: false}").into_event()
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
            div class="flex bg-slate-50 flex-col items-center justify-between h-screen w-screen"{
                div {}
                div data-signals="{sent : false}" class="flex flex-col items-center h-20 gap-2" {
                    button #hello type="button"
                        data-attr="{ disabled : $sent}"
                        data-on-click="@get('/test'); $sent = true;"
                        data-class="{ 'hover:bg-slate-200' : !$sent, 'hover:bg-red-50': $sent }"
                        class="border p-2 border-slate-800 rounded-md"
                        {"Press me"}
                    div class="" id="target" {}
                }
                div {}
                }
            }
    )
}
