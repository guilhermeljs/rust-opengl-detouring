use std::ffi::CString;

use windows::{core::PCSTR, Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress}};

#[derive(Debug)]
pub enum ParseError {
    InvalidCString(std::ffi::NulError),
    InvalidProcedure(String),
    GetModuleError(windows::core::Error)
}

pub struct Pointer(pub usize);

pub fn get_dll(dll: &str) -> Result<Pointer, ParseError> {
    let lib = CString::new(dll)
        .map_err(|e| ParseError::InvalidCString(e))?;

    unsafe {
        let handle = GetModuleHandleA(PCSTR(lib.as_ptr() as *const u8))
            .map_err(|e| ParseError::GetModuleError(e))?;

        Ok(Pointer(handle.0 as usize))
    }
}

pub fn get_dll_proc(dll: &str, proc: &str) -> Result<Pointer, ParseError>  {
    let lib = CString::new(dll)
        .map_err(|e| ParseError::InvalidCString(e))?;

    let procedure = CString::new(proc)
        .map_err(|e| ParseError::InvalidCString(e))?;

    unsafe {
        let handle = GetModuleHandleA(PCSTR(lib.as_ptr() as *const u8))
            .map_err(|e| ParseError::GetModuleError(e))?;

        let proc = GetProcAddress(handle, PCSTR(procedure.as_ptr() as *const u8))
            .ok_or(ParseError::InvalidProcedure(proc.to_string()))?;

        Ok(Pointer(proc as *mut u8 as usize))
    }
}