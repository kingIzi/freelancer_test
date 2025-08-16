
use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use dioxus::{dioxus_core::SpawnIfAsync, logger::tracing::instrument::WithSubscriber, prelude::*};
use dioxus_free_icons::{icons::bs_icons::BsX, Icon};
use dioxus_query::{prelude::{use_mutation, Captured, Mutation, MutationCapability, MutationReader, MutationStateData}, query::QueryCapability};
use validator::ValidateRequired;

use crate::{backend::{forms::{ResourceValues, Token}, utils}, frontend::form_builder::{FormControl, FormControlProps, FormGroup, Validator}};

use gloo_net::http::Request;
use http::StatusCode;
use web_sys::RequestCredentials;
use serde_json::Value;
use serde_json::Map;


//use axum_extra::extract::cookie::{Cookie, SameSite};


#[derive(Clone,Default)]
struct AuthRequests(Rc<RefCell<Value>>);

impl AuthRequests {
    async fn login(&self,body:& Value) -> Result<Value,ServerFnError> {
        let url = "http://127.0.0.1:8080/api/login";
        let result = Request::post(url)
        .credentials(RequestCredentials::Include)
        .json(body)?
        .send()
        .await?;
        let status = StatusCode::from_u16(result.status())
        .map_err(|_| ServerFnError::<String>::ServerError(StatusCode::INTERNAL_SERVER_ERROR.to_string())).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        match status {
            StatusCode::OK => {
                match result.json::<Value>().await {
                    std::result::Result::Ok(value) => std::result::Result::Ok(value),
                    Err(_) => Err(ServerFnError::ServerError(StatusCode::INTERNAL_SERVER_ERROR.to_string()))
                }
            },
            _ => Err(ServerFnError::ServerError(String::from(status.as_str())))
        }    
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
struct SignInRequest(Captured<AuthRequests>);

impl MutationCapability for SignInRequest {
    type Ok = Value;
    type Err = ServerFnError;
    type Keys = Value;
    
    async fn run(&self, body: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        self.0.login(body).await
    }
    
}

fn create_sign_in_form() -> FormGroup{
    let mut form = FormGroup::builder();
    form.add_control("password", FormControl::control(String::new(), vec![
        Validator::new("required", Validator::required()),
        Validator::new("invalidPhoneNumber", Validator::pattern(r"^\+255\s[67]\d{2}\s\d{3}\s\d{3}$"))
    ]));
    form
}

#[component]
pub fn PhoneNumberInput(form_control: FormControl) -> Element {
    let format_phone_number = move |evt: Event<FormData>,control: &mut FormControl| {
        let value = utils::format_phone_number(&evt.value());
        control.set_value(value);
    };
    let validate_phone_number = move |_,control: &mut FormControl| {
        control.validate();
    };
    let error_message = |error:&str| {
        match error {
            "required" => "Enter your mobile number",
            "invalidPhoneNumber" => "The mobile number you have entered is invalid",
            _ => ""
        }
    };
    let error = form_control.errors().first().map(|v| error_message(v.as_str()));
    rsx! {
        div { class: "flex flex-col w-full",
            input {
                class: "input w-full",
                placeholder: "(+255) XXX-XXX-XXX",
                r#type: "text",
                value: *form_control.get_raw_value(),
                oninput: move |evt| format_phone_number(evt, &mut form_control),
                onfocusout: move |evt| validate_phone_number(evt, &mut form_control),
            }
            div { class: "transition-all duration-200 relative flex flex-row justify-end items-center w-full h-fit",
                p {
                    class: format!(
                        "label text-right text-[var(--color-error)] transition-all duration-300 {}",
                        if error.is_some() { "opacity-100 h-fit" } else { "opacity-0 h-0" },
                    ),
                    "{error.unwrap_or_default()}"
                }
            }
        }
    }
}

fn clean_login_body(value:&mut Map<String,serde_json::Value>) -> Map<String,serde_json::Value> {
    let key = String::from("password");
    let password = value
    .remove(&key)
    .filter(|p| p.is_string())
    .and_then(|f| f.as_str().map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>()));                
    
    let password = password.unwrap_or(String::new());
    let password = serde_json::Value::String(password);
    value.insert(key, password);
    value.to_owned()
}

#[component]
pub fn SignPage() -> Element{
    let sign_in = use_mutation(Mutation::new(SignInRequest(Captured(AuthRequests::default()))));
    let mut form: Signal<Option<FormGroup>> = use_signal(|| None);
    let mut password: Signal<Option<Arc<Mutex<FormControl>>>> = use_signal(|| None);
    let mut alert_text: Signal<String> = use_signal(|| String::new());
    let mut alert_text_state: Signal<String> = use_signal(|| String::new());
    let navigator = use_navigator();
    let close_alert = move |evt: Event<MouseData>| {
        evt.prevent_default();
        evt.stop_propagation();
        alert_text.set(String::new());
        alert_text_state.set(String::new());
    }; 
    let on_submit = move || async move{
        let form_data = form().unwrap();
        let errors = form_data.validate_all();
        if errors.is_empty() {
            let body = serde_json::to_value(clean_login_body(&mut form_data.to_json())).unwrap();
            let sign_in_state = sign_in.mutate_async(body).await;
            let sign_in_state = sign_in_state.state();
            match sign_in_state.unwrap() {
                Ok(val) => {
                    alert_text.set(String::from("Logged in successfully!"));
                    alert_text_state.set(String::from("alert-success"));
                    gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
                    navigator.push("/admin");
                },
                Err(e) => {
                    match e {
                        ServerFnError::ServerError(code_str) => {
                            let status = code_str.parse::<u16>().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.as_u16());
                            match StatusCode::from_u16(status) {
                                Ok(StatusCode::BAD_REQUEST) => alert_text.set("400 Bad Request".to_string()),
                                Ok(StatusCode::NOT_FOUND) => alert_text.set("404 Not Found".to_string()),
                                Ok(StatusCode::INTERNAL_SERVER_ERROR) => alert_text.set("500 Internal Server Error".to_string()),
                                _ => alert_text.set(format!("Other error: {}", status)),
                            }
                        },
                        _ => alert_text.set("Unknown error".to_string()),
                    }
                    alert_text_state.set(String::from("alert-error"));
                },
            }
        }
    };
    let is_fetching_loading = move || {
        sign_in.read().state().is_loading()
    };
    use_effect(move || {
        form.set(Some(create_sign_in_form()));
        password.set(form().map(|f| f.get_control("password").unwrap()));
    });
    rsx! {
        div {
            "data-theme": "light",
            class: "w-screen h-screen grid grid-cols-1 lg:grid-cols-2",
            div { class: "w-full h-full hidden lg:block p-2",
                div { class: "w-full h-full relative",
                    img {
                        src: asset!("/assets/img/login-banner.png"),
                        class: "absolute w-full h-full object-cover rounded-4xl",
                    }
                }
            }
            div { class: "w-full h-full flex flex-col justify-between p-8 lg:p-2",
                div { class: "flex flex-row justify-start",
                    p { class: "uppercase text-2xl font-extrabold", "Freelancer" }
                }
                form {
                    id: "sign-in-form",
                    class: "flex flex-col space-y-4",
                    onsubmit: move |evt| async move {
                        on_submit().await;
                    },
                    div {
                        p { class: "text-4xl font-semibold", "Sign in or create an account." }
                        p { class: "text-lg", "Welcome back! Please enter your details." }
                    }
                    div {
                        class: format!(
                            "w-full duration-300 transition-all overflow-hidden {} relative",
                            if alert_text_state().is_empty() {
                                String::from("max-h-0")
                            } else {
                                String::from("max-h-40")
                            },
                        ),
                        div {
                            class: format!(
                                "alert alert-soft {} relative flex flex-row items-center justify-between w-full",
                                alert_text_state(),
                            ),
                            role: "alert",
                            span { "{alert_text()}" }
                            button {
                                class: "btn btn-circle btn-sm btn-ghost",
                                onclick: close_alert,
                                r#type: "button",
                                Icon {
                                    width: 24,
                                    height: 24,
                                    fill: "var(--color-secondary)",
                                    icon: BsX,
                                }
                            }
                        }
                    }
                    {
                        match password() {
                            Some(value) => {
                                rsx! {
                                    PhoneNumberInput { form_control: *value.lock().unwrap() }
                                }
                            }
                            None => {
                                rsx! {}
                            }
                        }
                    }
                    button {
                        form: "sign-in-form",
                        r#type: "submit",
                        class: "btn btn-secondary w-full",
                        disabled: is_fetching_loading(),
                        match is_fetching_loading() {
                            true => {
                                rsx! {
                                    span { class: "loading loading-spinner" }
                                    "Loading"
                                }
                            }
                            false => {
                                rsx! { "Become a member" }
                            }
                        }
                    }
                }
                p { class: "text-center w-full text-[var(--color-neutral)]",
                    "Copyright © 2025 - All right reserved"
                }
            }
        }
    }
}   