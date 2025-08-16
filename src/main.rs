mod backend;
mod frontend;


use chrono::round;
use dioxus::{dioxus_core::SpawnIfAsync, prelude::*};
#[cfg(feature = "server")]
use tokio::task::AbortHandle;


use crate::backend::forms::ResourceValues;
use crate::backend::utils;
use crate::backend::{forms::Token};
use crate::frontend::landing::Landing;
use crate::frontend::signin::SignPage;
use crate::frontend::admin_page::AdminPage;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    View,
    #[route("/signin")]
    SignPage,
    #[route("/admin")]
    AdminPage
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// fn main() {
//     dioxus::launch(App);
// }

fn main() {
    #[cfg(feature = "server")]
    tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {                
                launch_server(App).await;
            });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[cfg(feature = "server")]
async fn launch_server(component: fn() -> Element) {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::backend::{api::api, auths};

    let session_data = auths::auth_session::AuthSession::create_app_session().await;
    // let ip = dioxus::cli_config::fullstack_address_or_localhost().ip().to_owned();
    // let port = dioxus::cli_config::server_port().unwrap_or(dioxus::cli_config::server_port().unwrap().to_owned());
    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus::cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    let rest_api = axum::Router::new()
    .route("/register", axum::routing::post(api::Api::register_user))
    .route("/login", axum::routing::post(api::Api::login_user))
    .route("/is_authenticated", axum::routing::get(api::Api::is_authenticated));

    let router = axum::Router::new()
        .nest("/api", rest_api)
        .layer(session_data.layer.to_owned())
        .serve_dioxus_application(ServeConfigBuilder::default(), component)
        .into_make_service();

    axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal(session_data.deletion_task.abort_handle()))
    .await.unwrap();

    if let Err(err) = session_data.deletion_task.await {
        print!("Deletion task failed: {:?}", err);
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

use gloo_net::http::Request;
use web_sys::{window, RequestCredentials};


#[component]
fn View() -> Element{
    let mut is_authenticated = use_resource(move || async move {
        let url = "http://127.0.0.1:8080/api/is_authenticated";
        let result = Request::get(url)
        .credentials(RequestCredentials::Include)
        .send()
        .await;
        match result {
            Ok(resp) => {
                match resp.json::<Token>().await {
                    Ok(token) => Ok(token),
                    Err(err) => Err(format!("Failed to parse JSON: {}", err)),
                }
            }
            Err(err) => Err(format!("Request failed: {}", err)),
        }
    });
    let theme = utils::read_local_storage_value(utils::CURRENT_THEME);
    use_context_provider(|| ResourceValues { is_authenticated: is_authenticated, theme_mode: Signal::new(theme) });
    let mut context_resource = use_context::<ResourceValues>();
    rsx! {
        if let Some(response) = &*context_resource.is_authenticated.read() {
            match response {
                Ok(value) => {
                    rsx! {
                        div { "is authenticated" }
                    }
                }
                Err(_) => {
                    rsx! {
                        Landing {}
                    }
                }
            }
        } else {
            Landing {}
        }
    }
}

#[cfg(feature = "server")]
async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler")
    };
    
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {deletion_task_abort_handle.abort();}
        _ = terminate => {deletion_task_abort_handle.abort();}
    }
}