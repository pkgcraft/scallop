use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::{env, mem, process, ptr};

use scallop::bash::bash_main;

fn main() {
    let argv_strs: Vec<CString> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut argv_ptrs: Vec<*mut c_char> = argv_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    argv_ptrs.push(ptr::null_mut());
    let argv = argv_ptrs.as_ptr() as *mut _;
    let argc: c_int = argv_strs.len().try_into().unwrap();
    mem::forget(argv_strs);
    mem::forget(argv_ptrs);

    let env_strs: Vec<CString> = env::vars()
        .map(|(key, val)| format!("{}={}", key, val))
        .map(|s| CString::new(s).unwrap())
        .collect();
    let mut env_ptrs: Vec<*mut c_char> = env_strs.iter().map(|s| s.as_ptr() as *mut _).collect();
    env_ptrs.push(ptr::null_mut());
    let env = env_ptrs.as_ptr() as *mut _;
    mem::forget(env_strs);
    mem::forget(env_ptrs);

    let ret = unsafe { bash_main(argc, argv, env) };
    process::exit(ret)
}
