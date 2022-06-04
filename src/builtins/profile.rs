use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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
    let loops = Arc::new(AtomicUsize::new(0));
    let loops_t = loops.clone();
    eprintln!("profiling: {orig_cmd}");

    thread::spawn(move || -> Result<()> {
        // force success so the shell doesn't exit prematurely while profiling
        let cmd_str = format!("{orig_cmd} || :");
        let cmd = Command::new(cmd_str, None)?;
        loop {
            cmd.execute().ok();
            loops_t.fetch_add(1, Ordering::SeqCst);
        }
    });

    let start = Instant::now();
    thread::sleep(Duration::from_secs(3));
    let elapsed = start.elapsed();
    let loops = loops
        .load(Ordering::SeqCst)
        .try_into()
        .map_err(|_| Error::Base("failed converting loops".to_string()))?;

    match loops {
        0 => {
            eprintln!("elapsed {elapsed:?}, loops: 0");
        }
        n => {
            let per_loop = elapsed / loops;
            eprintln!("elapsed {elapsed:?}, loops: {n}, per loop: {per_loop:?}");
        }
    }

    Ok(ExecStatus::Success)
}

pub static BUILTIN: Builtin = Builtin {
    name: "profile",
    func: run,
    help: LONG_DOC,
    usage: "profile func arg1 arg2",
};
