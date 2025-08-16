use dioxus::prelude::*;

use dioxus::hooks::Resource;

use crate::backend::forms::{ResourceValues, Token};

#[cfg(feature = "server")]
use crate::backend::mongo_models::Docs::BaseUser;

#[cfg(feature = "server")]
use crate::backend::forms::Forms::AuthUserForm;
use crate::frontend::navbar;


#[component]
pub fn Landing() -> Element {
    let mut password = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut res = use_context::<ResourceValues>();
    let presentation_images = use_signal(|| vec![
        asset!("/assets/img/hair-4.jpeg"),
        asset!("/assets/img/hair-1.png"),
        asset!("/assets/img/hair-2.jpg"),
    ]);
    rsx! {
        div { "data-theme": "light", class: "w-screen h-screen flex flex-col",
            navbar::NavigationBar {}
            div { class: "px-5.5 py-8 space-y-2 w-full xl:w-1/2",
                div { class: "flex flex-col space-y-2 xl:h-fit",
                    p { class: "text-4xl lg:text-6xl", "Elevate your look with" }
                    p { class: "text-4xl lg:text-6xl", "authentic African styles." }
                }
                p { class: "text-lg",
                    "Celebrate African culture with our stunning hairstyles. From intricate braids to bold twists, we blend tradition with modern trends to enhance your unique style."
                }
                div { class: "py-0 flex flex-row items-center gap-2",
                    button { class: "btn btn-secondary", "Browse Hairstyles" }
                    button { class: "btn btn-ghost btn-outline", "Join" }
                }
            }
            div { class: "w-full h-full grid grid-cols-1 lg:grid-cols-3 p-8 gap-8",
                for presentation_image in presentation_images.read().iter() {
                    img {
                        src: "{presentation_image}",
                        loading: "lazy",
                        class: "w-full aspect-[5/3] object-cover rounded-2xl",
                    }
                }
            }
        }
    }
}