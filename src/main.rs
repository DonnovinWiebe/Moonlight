#![allow(clippy::elidable_lifetime_names)]
#![windows_subsystem = "windows"] // I follow the lifetime notation/elision suggestions in my editor (Zed).

use crate::state::app::App;

pub mod workspace;
pub mod state;
pub mod ui;

fn main() -> iced::Result {
    // sets the iced backend on linux manually - see below
    #[cfg(target_os = "linux")]
    // there have been some rendering issues on Fedora, and this fixed it
    unsafe { std::env::set_var("WGPU_BACKEND", "gl"); }

    // runs the app
    iced::application(App::new, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}