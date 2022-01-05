use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crossbeam_channel::tick;

use crate::builtins::Builtin;
use crate::command::Command;
use crate::Result;

static LONG_DOC: &str = "Profile a given function or command.";

#[doc = stringify!(LONG_DOC)]
pub(crate) fn run(args: &[&str]) -> Result<i32> {
    let cmd_str = args.join(" ");
    let cmd = Command::from_str(&cmd_str)?;

    let timeout = Arc::new(AtomicBool::new(false));
    let timeout2 = Arc::clone(&timeout);
    let (tx, rx) = mpsc::channel::<(Duration, u32)>();

    eprintln!("running: {}", cmd_str);
    let ticks = tick(Duration::from_secs(5));

    thread::spawn(move || {
        let mut loops = 0;
        let start = SystemTime::now();
        while !timeout.load(Ordering::Relaxed) {
            cmd.execute();
            loops += 1;
        }
        let elapsed = start.elapsed().expect("failed getting elapsed time");
        tx.send((elapsed, loops)).expect("channel transmit error");
    });

    ticks.recv().expect("channel receive error");
    timeout2.store(true, Ordering::Relaxed);
    let (elapsed, loops) = rx.recv().unwrap();
    let per_loop = elapsed / loops;
    eprintln!(
        "elapsed {:?}, loops: {}, per loop: {:?}",
        elapsed, loops, per_loop
    );

    Ok(0)
}

pub static BUILTIN: Builtin = Builtin {
    name: "profile",
    func: run,
    help: LONG_DOC,
    usage: "profile func arg1 arg2",
};
