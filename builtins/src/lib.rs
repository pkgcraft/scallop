use scallop::bash::bindings;
use scallop::bash::builtins::*;

#[cfg(feature = "pkgcraft")]
pub mod pkgcraft;

#[export_name = "profile_struct"]
static mut PROFILE_STRUCT: Option<bindings::Builtin> = None;

#[used]
#[link_section = ".init_array"]
static INITIALIZE_BUILTINS: extern "C" fn() = initialize_builtins;

#[no_mangle]
extern "C" fn initialize_builtins() {
    unsafe {
        PROFILE_STRUCT = Some(profile::BUILTIN.into());
    }
}
