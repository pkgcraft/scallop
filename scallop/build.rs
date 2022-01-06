use std::env;
use std::fs;
use std::path::PathBuf;

use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
struct BashCallback;

// rename bash data structures for consistency
impl ParseCallbacks for BashCallback {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        match original_item_name {
            // structs
            "word_desc" => Some("WordDesc".into()),
            "WORD_DESC" => Some("WordDesc".into()),
            "word_list" => Some("WordList".into()),
            "WORD_LIST" => Some("WordList".into()),
            "SHELL_VAR" => Some("ShellVar".into()),
            "ARRAY" => Some("Array".into()),
            "command" => Some("Command".into()),
            // global mutables
            "global_command" => Some("GLOBAL_COMMAND".into()),
            "this_command_name" => Some("CURRENT_COMMAND".into()),
            _ => None,
        }
    }
}

fn main() {
    let repo_dir_path = fs::canonicalize(format!("{}/../", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let repo_dir = repo_dir_path.to_str().unwrap();
    let scallop_build_dir = format!("{}/build", repo_dir);
    // link with scallop lib
    println!("cargo:rustc-link-search=native={}", scallop_build_dir);
    println!("cargo:rustc-link-lib=dylib=scallop");

    // used for static build
    //println!("cargo:rustc-link-search=native={}", bash_dir);
    //println!("cargo:rustc-link-lib=static=scallop");

    // https://github.com/rust-lang/cargo/issues/4895
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", scallop_build_dir);

    // generate bash bindings
    let bash_dir = format!("{}/bash", repo_dir);
    println!("cargo:rerun-if-changed=bash-wrapper.h");
    let bindings = bindgen::Builder::default()
        // add include dirs for clang
        .clang_arg(format!("-I{}", repo_dir))
        .clang_arg(format!("-I{}", bash_dir))
        .clang_arg(format!("-I{}/include", bash_dir))
        .clang_arg(format!("-I{}/builtins", bash_dir))
        .header("bash-wrapper.h")
        // command.h
        .allowlist_type("word_desc")
        .allowlist_type("word_list")
        .allowlist_var("global_command")
        .allowlist_function("copy_command")
        .allowlist_var("CMD_.*")
        // execute_command.h
        .allowlist_var("this_command_name")
        .allowlist_function("execute_command")
        // shell.h
        .allowlist_function("bash_main")
        // variables.h
        .allowlist_function("get_string_value")
        .allowlist_function("bind_variable")
        .allowlist_function("unbind_variable")
        .allowlist_function("find_variable")
        .allowlist_var("att_.*") // variable attributes
        // externs.h
        .allowlist_function("parse_command")
        // input.h
        .allowlist_function("with_input_from_string")
        .allowlist_function("push_stream")
        .allowlist_function("pop_stream")
        // dispose_cmd.h
        .allowlist_function("dispose_command")
        // builtins/common.h
        .allowlist_function("evalstring")
        .allowlist_var("SEVAL_.*")
        // subst.h
        .allowlist_var("ASS_.*")
        // array.h
        .allowlist_function("array_to_argv")
        // invalidate built crate whenever any included header file changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // mangle type names to expected values
        .parse_callbacks(Box::new(BashCallback))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bash-bindings.rs"))
        .expect("Couldn't write bindings!");
}
