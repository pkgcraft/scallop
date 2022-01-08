use scallop::bash;
use scallop::builtins;

#[export_name = "profile_struct"]
static mut PROFILE_STRUCT: Option<bash::Builtin> = None;

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
    // update struct pointers
    unsafe {
        PROFILE_STRUCT = Some(builtins::profile::BUILTIN.into());
    }

    // add builtins to known run() mapping
    builtins::update_run_map([&builtins::profile::BUILTIN])
}
