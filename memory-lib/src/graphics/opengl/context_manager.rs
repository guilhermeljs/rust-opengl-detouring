use std::{cell::OnceCell, sync::OnceLock};

use memory_sys::internal::logging::debug_log;
use windows::Win32::Graphics::{Gdi::HDC, OpenGL::{wglCreateContext, wglGetCurrentContext, wglMakeCurrent, wglShareLists, HGLRC}};

use crate::egui::pipeline::initialize_egui_glow;

use super::{errors::GLError, pipeline};

static mut ORIGINAL_CTX: OnceCell<HGLRC> = OnceCell::new();
static mut OVERLAY_CTX: OnceCell<HGLRC> = OnceCell::new();

static INITIALIZED: OnceLock<bool> = OnceLock::new();

/// --- Flux

fn initialize(hdc: HDC) -> Result<(), GLError> {
    debug_log("Initialize called");
    unsafe {
        create_shared_gl_context(hdc).map_err(|s| GLError::SharedGlContext(s))?;
    }

    pipeline::initialize(hdc);

    unsafe {
        swap_context(hdc, ToContext::GameContext)?;
    }

    Ok(())
}



fn render(hdc: HDC) -> Result<(), GLError> {

    
    unsafe { swap_context(hdc, ToContext::OverlayContext)?; }

    pipeline::render(hdc);

    unsafe { swap_context(hdc, ToContext::GameContext)?; }

    Ok(())
}

// --- Implementation

unsafe fn create_shared_gl_context(hdc: HDC) -> Result<(), String>{
    if ORIGINAL_CTX.get().is_some() && OVERLAY_CTX.get().is_some() {
        debug_log("Warning! Tried to initialize GL Context after it was already initialized.");
        return Ok(())
    }

    let original_ctx = wglGetCurrentContext();
    let new_ctx = wglCreateContext(hdc).map_err(|e| format!("Error while creating wgl_context {:?}", e))?;

    let _ = wglShareLists(new_ctx, original_ctx).inspect_err(|e| debug_log(format!("Error while calling wglShareLists {:?}", e).as_str()));
    wglMakeCurrent(hdc, new_ctx).map_err(|e| format!("Error while calling wglMakeCurrent {}", e))?;

    let _ = ORIGINAL_CTX.set(original_ctx);
    let _ = OVERLAY_CTX.set(new_ctx);
    debug_log("OpenGL context initialized");
    
    Ok(())
}



pub fn handle_swap_buffers(hdc: HDC) {
    if INITIALIZED.get().is_none() {
        let _ = initialize(hdc)
            .inspect_err(|e| debug_log(format!("Error while initializing GL pipeline {:?}", e).as_str()))
            .inspect(|_| INITIALIZED.set(true).expect("Tried to initialize an already initialized GL context"));

    }else {

        let _ = render(hdc)
            .inspect_err(|e| debug_log(format!("Failed to render {:?}", e).as_str()));

    }
}



#[derive(Debug)]
enum ToContext {
    OverlayContext,
    GameContext
}



unsafe fn swap_context(hdc: HDC, context: ToContext) -> Result<(), GLError> {
    let ctx = match context {
        ToContext::OverlayContext => OVERLAY_CTX.get()
            .ok_or(GLError::SwapContextError("Failed to get overlay context. Is context manager initialized?".to_string())),

        ToContext::GameContext => ORIGINAL_CTX.get()
            .ok_or(GLError::SwapContextError("Failed to get game context. Is context manager initialized?".to_string())),
    }?;

    unsafe {
        let current_ctx = wglGetCurrentContext();
        if current_ctx == *ctx {
            debug_log(format!("Warning: Tried to swap to the alreay context {:?}", context).as_str());
            return Ok(())
        }

        wglMakeCurrent(hdc, *ctx).map_err(
            |e| GLError::SwapContextError(format!("Error while calling wglMakeCurrent to our context {}", e)))?;
    }

    Ok(())
}