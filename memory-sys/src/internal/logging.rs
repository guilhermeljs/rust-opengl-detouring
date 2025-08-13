use std::ffi::CString;

use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;
use windows::core::PCSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK};

pub fn message_box(msg: &str) {
    let c_msg = CString::new(msg).unwrap();

    unsafe {
        let message = PCSTR(c_msg.as_ptr() as *const u8);
        let title = PCSTR(c_msg.as_ptr() as *const u8);
        MessageBoxA(None, message, title, MB_OK);
    }
}

pub fn debug_log(msg: &str) {
    let c_msg = CString::new(msg).unwrap();

    unsafe {
        OutputDebugStringA(PCSTR(c_msg.as_ptr() as *const u8));
    }
}