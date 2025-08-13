use std::{ffi::c_void, ptr, sync::OnceLock};

use windows::Win32::{Foundation::HANDLE, System::Memory::{VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE}};

use super::{library::{get_dll, get_dll_proc}, logging::debug_log};

static TRAMPOLINE: OnceLock<usize> = OnceLock::new();

#[no_mangle]
pub extern "C" fn teste(hdc: HANDLE) {
    debug_log("Hooked succesfully");

    debug_log("Hooked");


    unsafe {

        if let None = TRAMPOLINE.get() {
            debug_log("Trampoline is none");
        }else {
            debug_log("Trampoline found");
            let trampoline: extern "C" fn() = std::mem::transmute(TRAMPOLINE.get().unwrap());
            debug_log("Trampoline transmuted");

            trampoline();

            debug_log("Call after");
        }
    }
}
// Generates a 64-bit absolute jump instruction (`jmp rax`) for x86_64.
///
/// This function takes an absolute memory address (`absolute_location`) as `u64`
/// and returns the corresponding 12-byte machine code sequence:
///
/// ```text
/// 48 B8 <8-byte address> FF E0
/// ```
///
/// - `48 B8` — `mov rax, <address>`
/// - `<8-byte address>` — The little-endian representation of the absolute address.
/// - `FF E0` — `jmp rax`
///
/// # Parameters
/// - `absolute_location`: The target memory address to jump to.
///
/// # Returns
/// A `[u8; 12]` array containing the machine code for:
/// ```asm
/// mov rax, absolute_location
/// jmp rax
/// ```
///
/// # Example
/// ```rust
/// let jump_bytes = get_rax_jmp_x64(0x7FF6_1234_5678_9ABC);
/// // jump_bytes now holds the 12-byte sequence to jump to that address
/// ```
fn get_rax_jmp_x64(absolute_location: u64) -> [u8; 12] {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = absolute_location.to_le_bytes();
    [0x48, 0xB8, b0, b1, b2, b3, b4, b5, b6, b7, 0xFF, 0xE0]
}

pub fn create_generic_trampoline(original_location: usize, copy_len: usize) -> usize {
    let trampoline = unsafe {
        let jmp_location = (original_location + copy_len) as u64;
        debug_log(format!("Jmp location {}", jmp_location).as_str());
        let trampoline_ret = get_rax_jmp_x64(jmp_location);
        let trampoline_total_len = copy_len + trampoline_ret.len();

        let location = original_location as *const c_void;

        let memory_pointer = VirtualAlloc(
            None, 
            trampoline_total_len, 
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE
        );

        // copy the original bytes to the allocated memory
        ptr::copy_nonoverlapping(location, memory_pointer, copy_len);

        // copy the trampoline return to the memory pointer
        let trampoline_ret_len = trampoline_ret.len();
        let trampoline_ret = trampoline_ret.as_ptr();
        ptr::copy_nonoverlapping(trampoline_ret, memory_pointer.offset(copy_len as isize) as *mut u8, trampoline_ret_len);

        memory_pointer
    };

    trampoline as usize
}

/// Len needs to be atleast 13, for a x64 absolute jump using rax
/// 
/// **mov rax, {x64 abs pointer}** -> 10 bytes
/// 
/// **jmp rax** -> 2 bytes
/// 
/// **noop** -> if the len is more than 12, every byte will be a noop here.
/// 
/// 
pub fn hook(source: usize, hook_function: usize, len: usize) {
    if len < 12 {
        debug_log(format!("Error while hooking: len should be atleast 12, got {}", len).as_str());
        return;
    }

    let source_size = source;
    let source = source as *mut u8;

    let jmp = get_rax_jmp_x64(hook_function as u64);
    let jmp_ptr = jmp.as_ptr();

    let mut end_protection = PAGE_EXECUTE_READ;
    let mut old_protection = PAGE_EXECUTE_READ;
    debug_log(format!("Trying to hook {:02X?}", source_size).as_str());

    unsafe {
        let _ = VirtualProtect(source as *const c_void, len, PAGE_EXECUTE_READWRITE,  &mut old_protection);

        ptr::copy(jmp_ptr, source, jmp.len());
        debug_log(format!("Hooked {:02X?}", source_size).as_str());

        // fill the left slots with noop
        if len > jmp.len() {
            let noop = 0x90 as u8;

            ptr::write_bytes(source.offset(jmp.len() as isize), noop, len - jmp.len());
        }

        let _ = VirtualProtect(source as *const c_void , len, old_protection,  &mut end_protection);
    }

}

fn create_trampoline(original_bytes: [u8; 12], return_loc: usize) {
    
    let trampoline = unsafe {

        let ptr = VirtualAlloc(
            None,
            24,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE
        );
        
        std::ptr::copy_nonoverlapping(
            original_bytes.as_ptr(),
            ptr as *mut u8,
            12
        );
        
        let return_addr = return_loc + 10;
        let value = *(return_addr as *const u8);
        debug_log(format!("Return addr: {}", return_addr as isize).as_str());
        debug_log(format!("Trampoline Location: {}", ptr as isize).as_str());
        debug_log(format!("Opcode in Return: {:02X?}", value).as_str());
        //let jmp_offset = (return_addr as isize - (ptr as isize + 2)) as isize;

        /*debug_log(format!("Trampoline Offset: {}", jmp_offset).as_str());
        debug_log(format!("Trampoline Offset Byte: {:02X?}", jmp_offset.to_le_bytes()).as_str());*/
        *(ptr as *mut u8).add(12) = 0x48;
        *(ptr as *mut u8).add(13) = 0xB8;
        std::ptr::copy_nonoverlapping(
            &return_addr.to_le_bytes() as *const _ as *const u8,
            (ptr as *mut u8).add(14),
            8
        );

        *(ptr as *mut u8).add(22) = 0xFF;
        *(ptr as *mut u8).add(23) = 0xE0;
        
        ptr
    };

    debug_log("Trampoline created");
    let _ = TRAMPOLINE.set(trampoline as usize);
}

/*pub fn hook(location: usize) {
    let pointer = location as *mut c_void;

    debug_log("Hooking");

    let our_pointer = teste as *const ();
    let base_address = get_dll("memory_test.dll").expect("Error");

    let mut end_protection = PAGE_EXECUTE_READ;
    let mut old_protection = PAGE_EXECUTE_READ;
    unsafe {
        debug_log(format!("Target jmp: {}", base_address.0).as_str());
        debug_log(format!("Our fn location: {}", teste as isize).as_str());
        debug_log(format!("Location wgl_swap_buffers: {}", location as isize).as_str());
        
        let bytes = location as *const [u8; 12];
        let original_bytes = *bytes.clone();
        create_trampoline(original_bytes, location);
        
        let relative_offset = (teste as isize - location as isize) as isize;
        let relative_offset_valid = (teste as isize - location as isize) as isize;
        let relative_bytes = relative_offset.to_le_bytes();

        let target = (teste as u64).to_le_bytes();

        debug_log(format!("Relative offset {}", relative_offset).as_str());
        debug_log(format!("Relative bytes {:02X?}", relative_bytes).as_str());
        debug_log(format!("Real Relative bytes {}", relative_offset_valid).as_str());
        let _ = VirtualProtect(pointer , 12, PAGE_EXECUTE_READWRITE,  &mut old_protection);
        
        debug_log(format!("Removed virtual protection").as_str());


        let pointer = pointer as *mut u8;

        *pointer = 0x48;
        *(pointer.offset(1) as *mut u8) = 0xB8;

        *(pointer.offset(2) as *mut [u8; 8]) = target;

        *(pointer.offset(10) as *mut u8) = 0xFF;
        *(pointer.offset(11) as *mut u8) = 0xE0;

        let _ = VirtualProtect(location as *const c_void , 12, old_protection,  &mut end_protection);
    }
}*/