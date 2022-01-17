use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use autotools::Config;
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
            "builtin" => Some("Builtin".into()),
            // global mutables
            "global_command" => Some("GLOBAL_COMMAND".into()),
            "this_command_name" => Some("CURRENT_COMMAND".into()),
            "ifs_value" => Some("IFS".into()),
            _ => None,
        }
    }
}

fn main() {
    let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_dir = repo_path.to_str().unwrap();
    let bash_path = repo_path.join("bash");
    let bash_dir = bash_path.to_str().unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir_path = fs::canonicalize(format!("{}/../../../", out_dir)).unwrap();
    let target_dir = target_dir_path.to_str().unwrap();
    let bash_out_dir = &format!("{}/bash", target_dir);
    let bash_build_dir = &format!("{}/build", bash_out_dir);
    fs::create_dir_all(&bash_out_dir).unwrap();

    // TODO: Use the cc crate with some stub code to try compiling to see if the required dynamic
    // library exists before building our own.

    // build bash library if it doesn't exist
    let mut bash = Config::new(&bash_path);
    if !Path::new(&format!("{}/libbash.a", bash_build_dir)).exists() {
        bash.forbid("--disable-shared")
            .forbid("--enable-static")
            .enable("library", None)
            .disable("readline", None)
            .disable("history", None)
            .disable("bang-history", None)
            .disable("progcomp", None)
            .without("bash-malloc", None)
            .disable("mem-scramble", None)
            .disable("net-redirections", None)
            .disable("restricted", None)
            .disable("job-control", None);

        if !cfg!(feature = "nls") {
            bash.disable("nls", None);
        }

        // build static bash library
        bash.make_args(vec![format!("-j{}", num_cpus::get())])
            .make_target("libbash.a")
            .out_dir(&bash_out_dir)
            .build();
    }

    if cfg!(feature = "shared") {
        let meson_build_dir = &format!("{}/meson", target_dir);
        if !Path::new(&format!("{}/libscallop.so", meson_build_dir)).exists() {
            Command::new("meson")
                .args([
                    "setup",
                    meson_build_dir,
                    repo_dir,
                    &format!("-Dbash_libdir={}", bash_build_dir),
                ])
                .stdout(Stdio::inherit())
                .output()
                .expect("meson setup failed");
            Command::new("meson")
                .args(["compile", "-C", meson_build_dir, "-v"])
                .stdout(Stdio::inherit())
                .output()
                .expect("meson compile failed");
        }

        // use shared scallop library
        println!("cargo:rustc-link-search=native={}", meson_build_dir);
        println!("cargo:rustc-link-lib=dylib=scallop");

        // https://github.com/rust-lang/cargo/issues/4895
        println!("cargo:rustc-env=LD_LIBRARY_PATH={}", meson_build_dir);
    } else {
        // link statically with bash
        println!("cargo:rustc-link-search=native={}", bash_build_dir);
        println!("cargo:rustc-link-lib=static=bash");
    }

    // add bash symbols to scallop's dynamic symbol table
    // -- required for loading external builtins
    //println!("cargo:rustc-link-arg-bin=scallop=-rdynamic");

    // generate bash bindings
    println!("cargo:rerun-if-changed=bash-wrapper.h");
    let bindings = bindgen::Builder::default()
        // add include dirs for clang
        .clang_arg(format!("-I{}", bash_build_dir))
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
        .allowlist_function("lib_init")
        .allowlist_function("lib_reset")
        .allowlist_var("shell_name")
        .allowlist_var("EXECUTION_FAILURE")
        .allowlist_var("EXECUTION_SUCCESS")
        // variables.h
        .allowlist_function("get_string_value")
        .allowlist_function("bind_variable")
        .allowlist_function("bind_global_variable")
        .allowlist_function("unbind_variable")
        .allowlist_function("check_unbind_variable")
        .allowlist_function("find_variable")
        .allowlist_var("att_.*") // variable attributes
        // externs.h
        .allowlist_function("parse_command")
        .allowlist_function("strvec_dispose")
        // input.h
        .allowlist_function("with_input_from_string")
        .allowlist_function("push_stream")
        .allowlist_function("pop_stream")
        // dispose_cmd.h
        .allowlist_function("dispose_command")
        .allowlist_function("dispose_words")
        // builtins/common.h
        .allowlist_function("evalstring")
        .allowlist_function("source_file")
        .allowlist_function("register_builtins")
        .allowlist_function("builtin_address_internal")
        .allowlist_var("SEVAL_.*")
        // subst.h
        .allowlist_function("list_string")
        .allowlist_var("ifs_value")
        .allowlist_var("ASS_.*")
        // array.h
        .allowlist_function("array_to_argv")
        // builtins.h
        .blocklist_type("builtin")
        .allowlist_var("BUILTIN_ENABLED")
        .allowlist_var("STATIC_BUILTIN")
        .allowlist_var("ASSIGNMENT_BUILTIN")
        .allowlist_var("LOCALVAR_BUILTIN")
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
