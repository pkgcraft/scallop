use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::builtins::{Builtin, ExecStatus};
use crate::command::Command;
use crate::{Error, Result};

static LONG_DOC: &str = "Profile a given function or command.";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<ExecStatus> {
    if args.is_empty() {
        return Err(Error::Builtin("requires 1 or more args, got 0".into()));
    }

    let orig_cmd = args.join(" ");
    // force success so the shell doesn't exit prematurely while profiling
    let cmd_str = format!("{} || :", orig_cmd);
    let cmd = Command::new(cmd_str, None)?;

    let timeout = Arc::new(AtomicBool::new(false));
    let timeout2 = Arc::clone(&timeout);
    let mut loops = 0;

    eprintln!("profiling: {}", orig_cmd);

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        timeout2.store(true, Ordering::Relaxed);
    });

    let start = SystemTime::now();
    while !timeout.load(Ordering::Relaxed) {
        cmd.execute().ok();
        loops += 1;
    }
    let elapsed = start.elapsed().expect("failed getting elapsed time");
    let per_loop = elapsed / loops;
    eprintln!(
        "elapsed {:?}, loops: {}, per loop: {:?}",
        elapsed, loops, per_loop
    );

    Ok(ExecStatus::Success)
}

pub static BUILTIN: Builtin = Builtin {
    name: "profile",
    func: run,
    help: LONG_DOC,
    usage: "profile func arg1 arg2",
};
