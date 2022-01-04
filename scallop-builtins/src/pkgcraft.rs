use scallop::bindings;
use scallop::builtins::*;

#[export_name = "has_struct"]
static mut HAS_STRUCT: Option<bindings::Builtin> = None;
#[export_name = "hasv_struct"]
static mut HASV_STRUCT: Option<bindings::Builtin> = None;
#[export_name = "ver_cut_struct"]
static mut VER_CUT_STRUCT: Option<bindings::Builtin> = None;
#[export_name = "ver_rs_struct"]
static mut VER_RS_STRUCT: Option<bindings::Builtin> = None;
#[export_name = "ver_test_struct"]
static mut VER_TEST_STRUCT: Option<bindings::Builtin> = None;

#[cfg(target_os = "linux")]
#[used]
#[link_section = ".init_array"]
static INITIALIZE_PKGCRAFT_BUILTINS: extern "C" fn() = initialize_pkgcraft_builtins;

#[cfg(target_os = "macos")]
#[used]
#[link_section = "__DATA,__mod_init_func"]
static INITIALIZE_PKGCRAFT_BUILTINS: extern "C" fn() = initialize_pkgcraft_builtins;

#[no_mangle]
extern "C" fn initialize_pkgcraft_builtins() {
    unsafe {
        HAS_STRUCT = Some(has::BUILTIN.into());
        HASV_STRUCT = Some(hasv::BUILTIN.into());
        VER_CUT_STRUCT = Some(ver_cut::BUILTIN.into());
        VER_RS_STRUCT = Some(ver_rs::BUILTIN.into());
        VER_TEST_STRUCT = Some(ver_test::BUILTIN.into());
    }
}
