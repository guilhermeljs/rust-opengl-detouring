#[cfg(target_os = "windows")]
#[cfg_attr(feature = "internal", macro_export)]
macro_rules! dll_entrypoint {
    () => {
        #[no_mangle]
        pub extern "system" fn DllMain(
            hinst_dll: *mut std::ffi::c_void,
            fdw_reason: u32,
            _lpv_reserved: *mut std::ffi::c_void,
        ) -> i32 {
            if fdw_reason == 1 {
                main();
            }

            1
        }
    };
}

pub mod library;
pub mod logging;
pub mod detour;