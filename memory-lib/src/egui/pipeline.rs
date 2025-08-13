use std::{ffi::c_void, mem::zeroed, sync::{Arc, Mutex, OnceLock}};

use egui::{Context, Event, FullOutput, Modifiers, PointerButton, RawInput, Vec2};
use egui_glow::{glow::{self, HasContext}, Painter};
use memory_sys::internal::{library::get_dll_proc, logging::debug_log};
use windows::{core::PCSTR, Win32::{Graphics::{Gdi::HDC, OpenGL::{wglGetCurrentContext, wglGetProcAddress}}, UI::{Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON}, WindowsAndMessaging::GetCursorPos}}};

static CONTEXT: OnceLock<Context> = OnceLock::new();
static GLOW_CONTEXT: OnceLock<Arc<egui_glow::painter::Context>> = OnceLock::new();
static PAINTER: OnceLock<Mutex<Painter>> = OnceLock::new();
use windows::Win32::Foundation::POINT;


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

    // Atribui os eventos modificados ao RawInput
    input.events = events;

    input
}


pub fn render_egui(full_output: FullOutput) -> Result<(), String> {

    let ctx = CONTEXT.get().ok_or(format!("ERROR: Egui context was not initialized"))?;

    {
        let mut painter = PAINTER.get().ok_or(format!("ERROR: Painter was not initialized"))?
            .lock().map_err(|e| format!("Error while locking painter {}", e))?;
        
        let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        painter.paint_and_update_textures([1920, 1080], 1.0, &clipped_primitives, &full_output.textures_delta);
    }

    Ok(())

}

pub fn draw_egui<F>(draw: F) -> Result<FullOutput, String>
where
    F: Fn(&Context) {
    let ctx = CONTEXT.get().ok_or(format!("ERROR: Egui context was not initialized"))?;
    let raw_input = get_raw_input();
    let full_output = ctx.run(raw_input, |ctx| {
        draw(ctx);
    });

    Ok(full_output)
}


pub fn initialize_egui_glow(hdc: HDC)  {
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

    let ctx = egui::Context::default();

    let painter = Painter::new(glow_arc.clone(), "", None, false)
        .map_err(|e| format!("Error while creating painter {}", e))
        .inspect_err(|e| debug_log(format!("Error while creating painter {}", e).as_str()))
        .unwrap();

    unsafe {
        let error = glow_arc.get_error();
        debug_log(format!("Erro no GL: {}", error).as_str());
    }

    let _ = CONTEXT.set(ctx).map_err(|_| "Error setting Context, LOCK IS NOT NULL");
    let _ = GLOW_CONTEXT.set(glow_arc).map_err(|_| "Error setting Glow Context, LOCK IS NOT NULL");
    let _ = PAINTER.set(Mutex::new(painter)).map_err(|_| "Error setting Painter, LOCK IS NOT NULL");

}