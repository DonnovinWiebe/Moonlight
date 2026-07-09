use uuid::Uuid;

/// Various signals that allow the `App` to communicate with the ui.
#[derive(Debug, Clone)]
pub enum Signal {
    // initial app loading
    StartLoading,
    FinishedLoading,

    // node tree
    SelectNode(Uuid),
}