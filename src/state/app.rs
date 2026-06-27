use iced::{Subscription, Task, event, overlay::Element, widget::Text};

use crate::state::signal::Signal;

/// The central application container.
/// This holds the `Bank` and all ui/ux state information.
#[allow(clippy::struct_excessive_bools)] // This is more ergonomic than using enums for bool flags.
pub struct App {
}
impl App {
    // initializing
    /// Creates a new `App`.
    pub fn new() -> (App, Task<Signal>) {
        // returning the app
        (App {}, Task::done(Signal::StartLoading))
    }

    /// The tile of the `App`.
    #[must_use]
    pub fn title(&self) -> String {
        "Moonlight".to_string()
    }



    // running
    /// Updates the `App` based on a given `Signal`.
    #[allow(clippy::too_many_lines)] // This is going to be large since it is the central signal handler.
    pub fn update(&mut self, signal: Signal) -> Task<Signal> {
    
        // if the app loaded successfully, the app runs as normal
        match signal {
            Signal::StartLoading => {
                Task::done(Signal::FinishedLoading)
            },
            
            Signal::FinishedLoading => {
                Task::none()
            },
        }
    }
    
    /// Manages keybind input.
    pub fn subscription(&self) -> Subscription<Signal> {
        event::listen_with(|event, _status, _window| { None })
    }

    /// Renders the `App`.
    pub fn view<'a>(&'a self) -> Element<'a, Signal> {
        Text::new("Moonlight").into()
    }
}