use scallop::bash;
use scallop::builtins::*;

#[export_name = "assert_struct"]
static mut ASSERT_STRUCT: Option<bash::Builtin> = None;
#[export_name = "die_struct"]
static mut DIE_STRUCT: Option<bash::Builtin> = None;
#[export_name = "EXPORT_FUNCTIONS_struct"]
static mut EXPORT_FUNCTIONS_STRUCT: Option<bash::Builtin> = None;
#[export_name = "nonfatal_struct"]
static mut NONFATAL_STRUCT: Option<bash::Builtin> = None;
#[export_name = "has_struct"]
static mut HAS_STRUCT: Option<bash::Builtin> = None;
#[export_name = "hasv_struct"]
static mut HASV_STRUCT: Option<bash::Builtin> = None;
#[export_name = "ver_cut_struct"]
static mut VER_CUT_STRUCT: Option<bash::Builtin> = None;
#[export_name = "ver_rs_struct"]
static mut VER_RS_STRUCT: Option<bash::Builtin> = None;
#[export_name = "ver_test_struct"]
static mut VER_TEST_STRUCT: Option<bash::Builtin> = None;

#[cfg(target_os = "linux")]
#[used]
#[link_section = ".init_array"]
static INITIALIZE_PKG_BUILTINS: extern "C" fn() = initialize_pkg_builtins;

#[cfg(target_os = "macos")]
#[used]
#[link_section = "__DATA,__mod_init_func"]
static INITIALIZE_PKG_BUILTINS: extern "C" fn() = initialize_pkg_builtins;

#[no_mangle]
extern "C" fn initialize_pkg_builtins() {
    unsafe {
        ASSERT_STRUCT = Some(assert::BUILTIN.into());
        DIE_STRUCT = Some(die::BUILTIN.into());
        EXPORT_FUNCTIONS_STRUCT = Some(export_functions::BUILTIN.into());
        NONFATAL_STRUCT = Some(nonfatal::BUILTIN.into());
        HAS_STRUCT = Some(has::BUILTIN.into());
        HASV_STRUCT = Some(hasv::BUILTIN.into());
        VER_CUT_STRUCT = Some(ver_cut::BUILTIN.into());
        VER_RS_STRUCT = Some(ver_rs::BUILTIN.into());
        VER_TEST_STRUCT = Some(ver_test::BUILTIN.into());
    }
}
