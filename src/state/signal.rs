use uuid::Uuid;

/// Various signals that allow the `App` to communicate with the ui.
#[derive(Debug, Clone)]
pub enum Signal {
    // initial app loading
    StartLoading,
    FinishedLoading,

    // errors
    DismissErrors,

    // node tree
    SelectNode(Uuid),
}