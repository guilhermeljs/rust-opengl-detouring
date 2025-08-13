
use std::{cell::Cell, collections::VecDeque, sync::{Arc, Mutex, OnceLock}};

use windows::Win32::Graphics::{Gdi::HDC, OpenGL::{glBegin, glColor3f, glEnd, glVertex2f, GL_QUADS}};

use super::errors::PipelineError;

static RENDER_PIPELINE: OnceLock<Arc<Mutex<dyn Pipeline<HDC> + Send>>> = OnceLock::new();
static INITIALIZE_PIPELINE: OnceLock<Arc<Mutex<dyn Pipeline<HDC> + Send>>> = OnceLock::new();

pub trait Pipeline<Context> {
    fn run(&self, context: Context);
}

pub trait MutablePipeline<Context> {
    fn add_stage<F>(&mut self, stage: F)
    where
        F: Fn(Context) + 'static + Send;
}

pub struct GenericPipeline<Context> {
    stages: Vec<Box<dyn Fn(Context) + Send>>,
}

impl<Context> GenericPipeline<Context> {
    pub fn empty() -> Self {
        GenericPipeline {
            stages: Vec::new(),
        }
    }

    pub fn add_stage<F>(&mut self, stage: F)
    where
        F: Fn(Context) + 'static + Send,
    {
        self.stages.push(Box::new(stage));
    }
}

impl<Context> Pipeline<Context> for GenericPipeline<Context> where Context: Copy {
    fn run(&self, hdc: Context) {
        for stage in &self.stages {
            stage(hdc);
        }
    }
}

pub fn configure_render_pipeline<P>(pipeline: P) -> Result<(), PipelineError>
where
    P: Pipeline<HDC> + Send  + 'static
{
    RENDER_PIPELINE.set(Arc::new(Mutex::new(pipeline)))
        .map_err(|_| PipelineError::PipelineAlreadyConfigured("configure_render_pipeline is being called more than once".to_string()))?;
    
    Ok(())
}

pub fn configure_init_pipeline<P>(pipeline: P) -> Result<(), PipelineError>
where
    P: Pipeline<HDC> + Send  + 'static
{
    INITIALIZE_PIPELINE.set(Arc::new(Mutex::new(pipeline)))
        .map_err(|_| PipelineError::PipelineAlreadyConfigured("configure_init_pipeline is being called more than once".to_string()))?;

    Ok(())
}



// --- Crate only interfaces

pub(crate) fn render(hdc: HDC) {
    if let Some(pipeline)  = RENDER_PIPELINE.get() {
        let pipeline = pipeline.lock().expect("Failed to lock mutex RENDER_PIPELINE");
        pipeline.run(hdc);
    }
}

pub(crate) fn initialize(hdc: HDC) {
    if let Some(pipeline)  = INITIALIZE_PIPELINE.get() {
        let pipeline = pipeline.lock().expect("Failed to lock mutex INITIALIZE_PIPELINE");
        pipeline.run(hdc);
    }
}