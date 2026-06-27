/// Various signals that allow the `App` to communicate with the ui.
#[derive(Debug, Clone)]
pub enum Signal {
    StartLoading,
    FinishedLoading,
}