use scallop::bindings;
use scallop::builtins::*;

#[export_name = "profile_struct"]
static mut PROFILE_STRUCT: Option<bindings::Builtin> = None;

#[cfg(target_os = "linux")]
#[used]
#[link_section = ".init_array"]
static INITIALIZE_BUILTINS: extern "C" fn() = initialize_builtins;

#[cfg(target_os = "macos")]
#[used]
#[link_section = "__DATA,__mod_init_func"]
static INITIALIZE_BUILTINS: extern "C" fn() = initialize_builtins;

#[no_mangle]
extern "C" fn initialize_builtins() {
    unsafe {
        PROFILE_STRUCT = Some(profile::BUILTIN.into());
    }
}
