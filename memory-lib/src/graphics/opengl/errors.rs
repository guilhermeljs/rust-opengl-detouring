#[derive(Debug)]
pub enum GLError {
    SharedGlContext(String),
    RenderError,
    SwapContextError(String)
}

pub enum PipelineError {
    PipelineAlreadyConfigured(String)
}