/// Errors for the UI Builder.
#[derive(Debug)]
pub enum BuilderErrors<'a> {
    /// Error to be used when the Builder fails to get the current widget.
    GetCurrentWidgetError(&'a str),
}
