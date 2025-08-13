/*
-
IMPLEMENTATION MOVED TO graphics/opengl/hook (which is the new entrypoint for opengl hooks)

WARNING: This is legacy and unused code
-
*/

use std::cell::OnceCell;
use std::ffi::{c_void, CString};
use std::mem::zeroed;
use std::sync::Arc;
use std::{os::windows::raw::HANDLE, sync::OnceLock};
use egui::ahash::{HashMap, HashMapExt};
use egui::mutex::Mutex;
use egui::{Color32, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_glow::glow::HasContext;
use egui_glow::{glow, Painter};
use windows::core::PCSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::WindowFromDC;
use windows::Win32::Graphics::OpenGL::{glBegin, glColor3f, glEnd, glGetString, glVertex2f, wglGetProcAddress, wglMakeCurrent, SwapBuffers, GL_QUADS, GL_VERSION};
use windows::Win32::Graphics::{Gdi::HDC, OpenGL::{wglCreateContext, wglGetCurrentContext, wglShareLists, HGLRC}};

use memory_sys::internal::{detour::{create_generic_trampoline, hook}, library::get_dll_proc, logging::debug_log};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
type Opengl32Func = extern "C" fn(HDC) -> ();
static TRAMPOLINE: OnceLock<Opengl32Func> = OnceLock::new();
static STARTED: OnceLock<bool> = OnceLock::new(); 
static CONTEXT: OnceLock<Context> = OnceLock::new();
static GLOW_CONTEXT: OnceLock<Arc<egui_glow::painter::Context>> = OnceLock::new();
static PAINTER: OnceLock<Mutex<Painter>> = OnceLock::new();
static mut ORIGINAL_CTX: OnceCell<HGLRC> = OnceCell::new();
static mut OUR_CTX: OnceCell<HGLRC> = OnceCell::new();

fn wgl_swapbuffers_hooked(hdc: HDC) {
    if STARTED.get().is_none() {
        debug_log("Hooked succesfully");
        unsafe { 
            let _ = create_shared_gl_context(hdc).inspect_err(|e| debug_log(e));
            let _ = initialize_egui_glow(hdc).inspect_err(|e| debug_log(e));
        }
    }

    if STARTED.get().is_some() {
        let _ = try_render(hdc).inspect_err(|e| debug_log(e));
    }

    let trampoline: extern "C" fn(HDC) -> () = *TRAMPOLINE.get().unwrap();

    trampoline(hdc);
}

fn get_raw_input() -> RawInput {
    let mut point: POINT = unsafe { zeroed() };

    let position_success = unsafe { GetCursorPos(&mut point).is_ok() };

    let x = if position_success { point.x } else { 0 };
    let y = if position_success { point.y } else { 0 };

    let mouse_pos = if position_success {
        Some(Vec2::new(point.x as f32, point.y as f32))
    } else {
        None
    };


    let left_click = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
    let right_click = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) } < 0;

    if left_click || right_click {
        debug_log(format!("Clicked {} {}", left_click, right_click).as_str());
    }

    let mut input = RawInput::default();

    let mut events = Vec::new();

    if let Some(pos) = mouse_pos {
        events.push(Event::PointerMoved(pos.to_pos2()));

        events.push(Event::PointerButton {
            pos: pos.to_pos2(),
            button: PointerButton::Primary,
            pressed: left_click,
            modifiers: Modifiers::default(),
        });

        events.push(Event::PointerButton {
            pos: pos.to_pos2(),
            button: PointerButton::Secondary,
            pressed: right_click,
            modifiers: Modifiers::default(),
        });
    }

    input.events = events;

    input
}

fn initialize_egui_glow(hdc: HDC) -> Result<(), String> {
    debug_log("Creating glow loader");
    let glow = unsafe {
        glow::Context::from_loader_function(|s| {
            let addr = wglGetProcAddress(PCSTR(s.as_ptr() as *const u8));
            if let Some(addr) = addr {
                return addr as *const c_void;
            }

            match get_dll_proc("OPENGL32.dll", s) {
                Ok(addr) => {
                    addr.0 as *const c_void
                },
                Err(_e) => {
                    debug_log(format!("Error while trying to get opengl procedure -> {}", s).as_str());
                    return std::ptr::null()
                }
            }
        })
    };

   let glow_arc = Arc::new(glow);

    debug_log("Creating egui context");
    let ctx = egui::Context::default();
    debug_log("Creating painter");
    let painter = Painter::new(glow_arc.clone(), "", None, false).map_err(|e| format!("Error while creating painter {}", e))?;

    let _ = CONTEXT.set(ctx).map_err(|_| "Error setting Context, LOCK IS NOT NULL")?;
    let _ = GLOW_CONTEXT.set(glow_arc).map_err(|_| "Error setting Glow Context, LOCK IS NOT NULL")?;
    let _ = PAINTER.set(Mutex::new(painter)).map_err(|_| "Error setting Painter, LOCK IS NOT NULL")?;

    Ok(())
}

fn try_render(hdc: HDC) -> Result<(), String> {
    if STARTED.get().is_some() {
        
        unsafe {
            let new_ctx = OUR_CTX.get().unwrap();
            wglMakeCurrent(hdc, *new_ctx).map_err(|e| format!("Error while calling wglMakeCurrent {}", e))?;
        }

        /*unsafe {
            glBegin(GL_QUADS);
            glColor3f(0.0, 1.0, 0.0);
            glVertex2f(-0.5, -0.5);
            glVertex2f( 0.5, -0.5);
            glVertex2f( 0.5,  0.5);
            glVertex2f(-0.5,  0.5);
            glEnd();
        }*/

        let ctx = CONTEXT.get().ok_or(format!("ERROR: Egui context was not initialized"))?;
        let glow_context = GLOW_CONTEXT.get().ok_or(format!("ERROR: Glow context was not initialized"))?;

       /*unsafe {
            glow_context.clear_color(0.0, 0.0, 0.0, 1.0);
            glow_context.as_ref().clear(glow::COLOR_BUFFER_BIT);
        }*/

        //let raw_input = egui::RawInput::default();
        let raw_input = get_raw_input();
        let full_output = ctx.run(raw_input, |ctx| {
            egui::Window::new("Minha Janela").current_pos(egui::pos2(100.0, 500.0)).show(ctx, |ui| {
                ui.label("Olá, mundo!");
                if ui.button("Clique aqui").clicked() {
                    println!("Botão pressionado!");
                }
            });
        });

        {
            let mut painter = PAINTER.get().ok_or(format!("ERROR: Painter was not initialized"))?
                .lock();
            
            let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
            painter.paint_and_update_textures([1920, 1080], 1.0, &clipped_primitives, &full_output.textures_delta);
        }

        unsafe {
            let old_ctx = ORIGINAL_CTX.get().unwrap();
            wglMakeCurrent(hdc, *old_ctx).map_err(|e| format!("Error while calling wglMakeCurrent {}", e))?;
        }

        return Ok(());
    }

    Err(String::from("Failed to render, context was not initialized"))
}

unsafe fn create_shared_gl_context(hdc: HDC) -> Result<HGLRC, String>{
    debug_log("Creating shared context");
    let original_ctx = wglGetCurrentContext();
    let new_ctx = wglCreateContext(hdc).map_err(|e| format!("Error while creating wgl_context {:?}", e))?;

    let _ = ORIGINAL_CTX.set(original_ctx);
    let _ = OUR_CTX.set(new_ctx);

    let _ = wglShareLists(new_ctx, original_ctx).inspect_err(|e| debug_log(format!("Error while calling wglShareLists {:?}", e).as_str()));
    wglMakeCurrent(hdc, new_ctx).map_err(|e| format!("Error while calling wglMakeCurrent {}", e))?;

    /*let string = CString::from_raw(glGetString(GL_VERSION) as *mut i8);

    debug_log(format!("OpenGL Version: {}", string.to_str().unwrap()).as_str());*/
    /* 
    debug_log("Creating glow loader");
    let glow = unsafe {
        glow::Context::from_loader_function(|s| {
            let addr = wglGetProcAddress(PCSTR(s.as_ptr() as *const u8));
            if let Some(addr) = addr {
                return addr as *const c_void;
            }

            match get_dll_proc("OPENGL32.dll", s) {
                Ok(addr) => {
                    addr.0 as *const c_void
                },
                Err(_e) => {
                    debug_log(format!("Error while trying to get opengl procedure -> {}", s).as_str());
                    return std::ptr::null()
                }
            }
        })
    };

   let glow_arc = Arc::new(glow);

    debug_log("Creating egui context");
    let ctx = egui::Context::default();
    debug_log("Creating painter");
    let painter = Painter::new(glow_arc.clone(), "", None, false).map_err(|e| format!("Error while creating painter {}", e))?;

    let _ = CONTEXT.set(ctx).map_err(|_| "Error setting Context, LOCK IS NOT NULL")?;
    let _ = GLOW_CONTEXT.set(glow_arc).map_err(|_| "Error setting Glow Context, LOCK IS NOT NULL")?;
    let _ = PAINTER.set(Mutex::new(painter)).map_err(|_| "Error setting Painter, LOCK IS NOT NULL")?;*/

    debug_log("OpenGL context initialized");
    let _ = STARTED.set(true);
    
    Ok(original_ctx)
}

pub fn hook_opengl() {
    let pointer = get_dll_proc("OPENGL32.DLL", "wglSwapBuffers").unwrap();

    let location = create_generic_trampoline(pointer.0, 15);
    debug_log(format!("Trampoline created at: {}" , location).as_str());

    unsafe {
        let _ = TRAMPOLINE.set(std::mem::transmute(location));
    }
    hook(pointer.0, wgl_swapbuffers_hooked as usize, 15);
}
