//! A stand-in for Apple's C compiler and archiver, for `check_on_windows.sh`.
//!
//! Three crates the game is built from compile a little C for the iPhone as
//! they are built (blake3, ring, tracing-oslog), which wants Apple's
//! compiler and SDK, and this PC has neither. `cargo check` never links, so
//! what they compile only has to be there: this makes an empty object for
//! every compile and an empty archive for every `ar`, and says it is clang
//! when the `cc` crate asks what compiler it is. Built with `rustc` alone;
//! it is part of no crate and of no app.

use std::{env, fs, path::Path};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    // The `cc` crate asking what kind of compiler this is.
    if args.iter().any(|arg| arg == "-E") {
        println!("#pragma message \"clang\"");
        return;
    }
    // A compile: `... -o <object> ...`.
    let mut made = false;
    let mut at = 0;
    while at < args.len() {
        if args[at] == "-o" && at + 1 < args.len() {
            let _ = fs::write(&args[at + 1], b"");
            made = true;
            at += 1;
        }
        at += 1;
    }
    if made {
        return;
    }
    // An archive: `<letters> <archive> [objects...]`.
    if args.len() >= 2 && args[0].chars().all(|letter| letter.is_ascii_alphabetic()) && !Path::new(&args[1]).exists() {
        let _ = fs::write(&args[1], b"!<arch>\n");
    }
}
