use scallop::bash::builtins::Builtin;

#[export_name = "has_struct"]
static mut HAS_STRUCT: Option<Builtin> = None;
#[export_name = "hasv_struct"]
static mut HASV_STRUCT: Option<Builtin> = None;
#[export_name = "ver_rs_struct"]
static mut VER_RS_STRUCT: Option<Builtin> = None;

#[used]
#[link_section = ".init_array"]
static INITIALIZE_BUILTINS: extern "C" fn() = initialize_builtins;

#[no_mangle]
extern "C" fn initialize_builtins() {
    unsafe {
        HAS_STRUCT = Some(Builtin::register("has"));
        HASV_STRUCT = Some(Builtin::register("hasv"));
        VER_RS_STRUCT = Some(Builtin::register("ver_rs"));
    }
}
