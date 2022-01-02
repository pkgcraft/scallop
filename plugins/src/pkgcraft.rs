use scallop::bash::builtins::Builtin;

#[export_name = "ver_cut_struct"]
static mut VER_CUT_STRUCT: Option<Builtin> = None;
#[export_name = "ver_rs_struct"]
static mut VER_RS_STRUCT: Option<Builtin> = None;
#[export_name = "ver_test_struct"]
static mut VER_TEST_STRUCT: Option<Builtin> = None;

#[used]
#[link_section = ".init_array"]
static INITIALIZE_PKGCRAFT_BUILTINS: extern "C" fn() = initialize_pkgcraft_builtins;

#[no_mangle]
extern "C" fn initialize_pkgcraft_builtins() {
    unsafe {
        VER_CUT_STRUCT = Some(Builtin::register("ver_cut"));
        VER_RS_STRUCT = Some(Builtin::register("ver_rs"));
        VER_TEST_STRUCT = Some(Builtin::register("ver_test"));
    }
}
