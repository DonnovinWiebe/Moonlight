use iced::{Element, Subscription, Task, Theme, event, widget::Text};
use materialui::{components::ThemeProvider, material::MaterialThemes};

use crate::{state::signal::Signal, workspace::tree::Tree};

/// The central application container.
/// This holds the `Bank` and all ui/ux state information.
#[allow(clippy::struct_excessive_bools)] // This is more ergonomic than using enums for bool flags.
pub struct App {
    // basic app state
    material_theme: MaterialThemes,
    application_failures: Vec<String>,
    theme: Theme,

    // projects
    current_tree: Option<usize>,
    orchard: Vec<Tree>,
}
impl ThemeProvider for App {
    /// Gets the current `MaterialTheme'.
    fn material_theme(&self) -> MaterialThemes {
        self.material_theme
    }
}
impl App {
    // initializing
    /// Creates a new `App`.
    pub fn new() -> (App, Task<Signal>) {
        let app = App {
            // basic app state
            material_theme: MaterialThemes::Midnight,
            application_failures: Vec::new(),
            theme: MaterialThemes::Midnight.generate_iced_palette(),

            // projects
            current_tree: None,
            orchard: Vec::new(),
        };
        
        (app, Task::done(Signal::StartLoading))
    }



    // basic getters
    /// The tile of the `App`.
    #[must_use]
    pub fn title(&self) -> String {
        "Moonlight".to_string()
    }
    
    /// Gets the current `Theme`.
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Gets the current `Tree`.
    pub fn get_current_tree(&self) -> Option<&Tree> {
        match &self.current_tree {
            Some(i) => { Some(&self.orchard[*i]) }
            None => { None }
        }
    }



    // running
    /// Updates the `App` based on a given `Signal`.
    #[allow(clippy::too_many_lines)] // This is going to be large since it is the central signal handler.
    pub fn update(&mut self, signal: Signal) -> Task<Signal> {
    
        // if the app loaded successfully, the app runs as normal
        match signal {
            // initial app loading
            Signal::StartLoading => {
                Task::done(Signal::FinishedLoading)
            },
            
            Signal::FinishedLoading => {
                Task::none()
            },


            
            // node tree
            Signal::SelectNode(Uuid) => {
                Task::none()
            }
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