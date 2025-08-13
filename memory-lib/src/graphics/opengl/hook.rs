use std::sync::OnceLock;

use memory_sys::internal::{detour::{create_generic_trampoline, hook}, library::get_dll_proc, logging::debug_log};
use windows::Win32::Graphics::Gdi::HDC;

use super::context_manager;

type GLTrampoline = extern "C" fn(HDC) -> ();

static TRAMPOLINE: OnceLock<GLTrampoline> = OnceLock::new();

fn wgl_swapbuffers_hooked(hdc: HDC) {
    context_manager::handle_swap_buffers(hdc);

    let trampoline: extern "C" fn(HDC) -> () = *TRAMPOLINE.get().unwrap();

    trampoline(hdc);
}


pub fn hook_opengl() {
    let pointer = get_dll_proc("OPENGL32.DLL", "wglSwapBuffers").unwrap();

    let location = create_generic_trampoline(pointer.0, 15);
    debug_log(format!("Trampoline created at {}", location).as_str());

    unsafe {
        let _ = TRAMPOLINE.set(std::mem::transmute(location));
    }
    hook(pointer.0, wgl_swapbuffers_hooked as usize, 15);
}