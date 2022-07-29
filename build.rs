use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
            "temporary_env" => Some("TEMPORARY_ENV".into()),
            "ifs_value" => Some("IFS".into()),
            "shell_builtins" => Some("SHELL_BUILTINS".into()),
            "num_shell_builtins" => Some("NUM_SHELL_BUILTINS".into()),
            "subshell_environment" => Some("SUBSHELL_ENVIRONMENT".into()),
            // functions
            "get_minus_o_opts" => Some("get_set_options".into()),
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
    // job control support is required for $PIPESTATUS
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
            .disable("restricted", None);

        if !cfg!(feature = "nls") {
            bash.disable("nls", None);
        }

        // build static bash library
        bash.make_args(vec![format!("-j{}", num_cpus::get())])
            .make_target("libbash.a")
            .out_dir(&bash_out_dir)
            .build();
    }

    if !cfg!(feature = "plugin") {
        // link statically with bash
        println!("cargo:rustc-link-search=native={}", bash_build_dir);
        println!("cargo:rustc-link-lib=static=bash");
    }

    // `cargo llvm-cov` currently appears to have somewhat naive object detection and erroneously
    // includes the config.status file causing it to error out
    let config_status = PathBuf::from(bash_build_dir).join("config.status");
    if config_status.exists() {
        fs::remove_file(config_status).expect("failed removing config.status file");
    }

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
        // execute_cmd.h
        .allowlist_var("this_command_name")
        .allowlist_function("execute_command")
        .allowlist_function("execute_shell_function")
        // shell.h
        .allowlist_function("bash_main")
        .allowlist_function("lib_error_handlers")
        .allowlist_function("lib_init")
        .allowlist_function("lib_reset")
        .allowlist_function("set_shell_name")
        .allowlist_var("shell_name")
        .allowlist_var("subshell_environment")
        .allowlist_var("EXECUTION_FAILURE")
        .allowlist_var("EXECUTION_SUCCESS")
        .allowlist_var("EX_LONGJMP")
        // error.h
        .allowlist_function("shm_error")
        // variables.h
        .allowlist_function("get_string_value")
        .allowlist_function("bind_variable")
        .allowlist_function("bind_global_variable")
        .allowlist_function("unbind_variable")
        .allowlist_function("check_unbind_variable")
        .allowlist_function("find_function")
        .allowlist_function("find_variable")
        .allowlist_function("push_context")
        .allowlist_function("pop_context")
        .allowlist_var("temporary_env")
        .allowlist_var("att_.*") // variable attributes
        // externs.h
        .allowlist_function("parse_command")
        .allowlist_function("strvec_dispose")
        .allowlist_function("strvec_to_word_list")
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
        .allowlist_function("get_minus_o_opts")
        .allowlist_function("get_shopt_options")
        .allowlist_var("SEVAL_.*")
        // subst.h
        .allowlist_function("expand_string_to_string")
        .allowlist_function("list_string")
        .allowlist_var("ifs_value")
        .allowlist_var("ASS_.*")
        // array.h
        .allowlist_function("array_to_argv")
        // builtins.h
        .allowlist_var("BUILTIN_ENABLED")
        .allowlist_var("STATIC_BUILTIN")
        .allowlist_var("ASSIGNMENT_BUILTIN")
        .allowlist_var("LOCALVAR_BUILTIN")
        .allowlist_var("num_shell_builtins")
        .allowlist_var("shell_builtins")
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
