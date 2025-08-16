
use std::ops::Deref;

use dioxus::prelude::*;


#[derive(Default,Clone)]
struct NavItem {
    label: String,
    route: String
}

#[component]
pub fn NavigationBar() -> Element {
    let nav_items = use_signal(|| vec![NavItem {
        label: String::from("About Us"),
        route: String::from("#")
    }, NavItem {
        label: String::from("Become a member"),
        route: String::from("/signin")
    }]);
    let navigator = use_navigator();
    let change_route = |e:&str,nav:&Navigator| {
        nav.push(e);
    };
    rsx! {
        div { class: "navbar bg-base-100 shadow-sm",
            div { class: "navbar-start",
                div { class: "dropdown",
                    div {
                        class: "btn btn-ghost lg:hidden",
                        role: "button",
                        tabindex: "0",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            xmlns: "http://www.w3.org/2000/svg",
                            path {
                                d: "M4 6h16M4 12h8m-8 6h16",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                            }
                        }
                    }
                    ul {
                        class: "menu menu-sm dropdown-content bg-base-100 rounded-box z-1 mt-3 w-52 p-2 shadow",
                        tabindex: "0",
                        {
                            nav_items
                                .read()
                                .iter()
                                .map(|nav_item| {
                                    let route = nav_item.route.clone();
                                    let label = nav_item.label.clone();
                                    rsx! {
                                        li {
                                            a { onclick: move |_| change_route(&route, &navigator), "{label}" }
                                        }
                                    }
                                })
                        }
                    }
                }
                a { href: "/", class: "btn btn-ghost text-xl", "Freelancer" }
            }
            div { class: "navbar-center hidden lg:flex",
                ul { class: "menu menu-horizontal px-1",
                    {
                        nav_items
                            .read()
                            .iter()
                            .map(|nav_item| {
                                let route = nav_item.route.clone();
                                let label = nav_item.label.clone();
                                rsx! {
                                    li {
                                        a { onclick: move |_| change_route(&route, &navigator), "{label}" }
                                    }
                                }
                            })
                    }
                }
            }
            div { class: "navbar-end",
                button { class: "btn btn-ghost btn-outline", "Join" }
            }
        }
    }
}