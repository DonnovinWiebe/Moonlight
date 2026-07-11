use iced::{Element, Subscription, Task, Theme, event, widget::Text};
use materialui::{components::{PageProvider, ThemeProvider}, material::MaterialThemes};

use crate::{state::{signal::Signal}, workspace::tree::Tree};



/// The pages available in the `App`.
pub enum Pages {
    Project,
}



/// The central application container.
/// This holds the `Bank` and all ui/ux state information.
#[allow(clippy::struct_excessive_bools)] // This is more ergonomic than using enums for bool flags.
pub struct App {
    // basic app state
    theme: Theme,
    material_theme: MaterialThemes,
    app_errors: Vec<String>,
    page: Pages,

    // projects
    current_tree: Option<usize>,
    orchard: Vec<Tree>,
}
impl PageProvider for App {
    fn page_name(&self) -> &str {
        match self.page {
            Pages::Project => { "Project Space" }
        }
    }

    fn page_icon(&self) -> &str {
        match self.page {
            Pages::Project => { "table-cells" }
        }
    }
}
impl ThemeProvider for App {
    fn material_theme(&self) -> MaterialThemes {
        self.material_theme
    }
}
impl App {
    // initializing
    /// Creates a new `App`.
    #[must_use]
    pub fn new() -> (App, Task<Signal>) {
        let app = App {
            // state
            theme: MaterialThemes::Midnight.generate_iced_palette(),
            material_theme: MaterialThemes::Midnight,
            app_errors: Vec::new(),
            page: Pages::Project,

            // projects
            current_tree: None,
            orchard: Vec::new(),
        };
        
        (app, Task::done(Signal::StartLoading))
    }



    // app info getters
    /// The tile of the `App`.
    #[must_use]
    pub fn title(&self) -> String {
        "Moonlight".to_string()
    }


    
    // state getters
    /// Gets the current `iced::Theme`.
    #[must_use]
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Gets the `app_errors`.
    #[must_use]
    pub fn get_app_errors(&self) -> &Vec<String> {
        &self.app_errors
    }


    
    // project getters
    /// Gets the current `Tree`.
    #[must_use]
    pub fn get_current_tree(&self) -> Option<&Tree> {
        match &self.current_tree {
            Some(i) => { Some(&self.orchard[*i]) }
            None => { None }
        }
    }



    // running
    /// Updates the `App` based on a given `Signal`.
    #[must_use]
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


            
            // errors
            Signal::DismissErrors => {
                self.app_errors.clear();
                Task::none()
            },


            
            // node tree
            Signal::SelectNode(Uuid) => {
                Task::none()
            },
        }
    }
    
    /// Manages keybind input.
    #[must_use]
    pub fn subscription(&self) -> Subscription<Signal> {
        event::listen_with(|event, _status, _window| { None })
    }

    /// Renders the `App`.
    #[must_use]
    pub fn view<'a>(&'a self) -> Element<'a, Signal> {
        Text::new("Moonlight").into()
    }
}