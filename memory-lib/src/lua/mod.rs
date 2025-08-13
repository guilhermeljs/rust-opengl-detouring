use std::{ffi::{c_char, c_void, CString}, ops::Add, os::raw::c_int, sync::OnceLock};

use memory_sys::internal::{detour::{create_generic_trampoline, hook}, logging::debug_log};
use windows::{core::PCSTR, Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleA}};


type TLuaLoadBuffer = unsafe extern "cdecl" fn(lua_state: *mut c_int, buff: *const c_char, sz: usize, name: *const c_char) -> c_int;
static TRAMPOLINE: OnceLock<TLuaLoadBuffer> = OnceLock::new();

type TLuaPCall = unsafe extern "cdecl" fn(lua_state: *mut c_int, nargs: *const c_int, nresults: c_int, errFunc: *const c_int) -> c_int;
static TRAMPOLINE_PCALL: OnceLock<TLuaPCall> = OnceLock::new();

type TLuaDoString = unsafe extern "C" fn(param1: *const c_char, param2: *const c_void) -> c_int;
static TAMPOLINE_DOSTRING: OnceLock<TLuaDoString> = OnceLock::new();

fn hooked_loadbuffer(lua_state: *mut c_int, buff: *const c_char, sz: usize, descr: *const c_char) {
    debug_log("Hooked load buffer");

    unsafe {
        let trampoline: TLuaLoadBuffer = *TRAMPOLINE.get().unwrap();

        trampoline(lua_state, buff, sz, descr);
    }
}

fn hooked_pcall(lua_state: *mut c_int, nargs: *const c_int, nresults: c_int, err_func: *const c_int) {
    debug_log("Hooked pcall buffer");

    unsafe {
        let trampoline: TLuaPCall = *TRAMPOLINE_PCALL.get().unwrap();

        trampoline(lua_state, nargs, nresults, err_func);
    }
}


fn get_base_address(module_name: &str) -> Option<HMODULE> {
    unsafe {
        let c_string = CString::new(module_name).unwrap();
        let module_handle = GetModuleHandleA(PCSTR(c_string.as_ptr() as *const u8));

        module_handle.ok()
    }
}


pub fn hook_lua() {
    /*let base_addr = get_base_address("gameclient.exe");
    let loadbuffer_offset = 0xCC9170;
    let pcall_offset = 0xCC82B0;

    if let Some(base_addr) = base_addr {
        let addr = base_addr.0 as usize;

        let addr = addr.add(loadbuffer_offset);
        let location = create_generic_trampoline(addr as usize, 12);
        debug_log(format!("Lua trampoline created at {}", location).as_str());
    
        unsafe {

            let _ = TRAMPOLINE.set(std::mem::transmute(location));
        }
        hook(addr, hooked_loadbuffer as usize, 12);


        
        let addr = (base_addr.0 as usize).add(pcall_offset);
        let pcall_trampoline = create_generic_trampoline(addr as usize, 19);
        debug_log(format!("Lua pcall trampoline created at {}", pcall_trampoline).as_str());
        unsafe {
            let _ = TRAMPOLINE_PCALL.set(std::mem::transmute(pcall_trampoline));
        }
        hook(addr, hooked_pcall as usize, 19);
    }*/
}