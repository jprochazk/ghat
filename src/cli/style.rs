use std::fmt::Display;
use std::io::{IsTerminal, Write};

/// Print a Cargo-style status line: a right-aligned bold green label followed by a message.
///
/// ```text
///    Checking workflow definitions
///   Evaluating workflow definitions
///        Wrote .github/workflows/generated_ci.yaml
///     Finished generated 2 workflows in 0.42s
/// ```
pub fn status(label: impl Display, message: impl Display) {
    let stderr = std::io::stderr();
    let use_color = stderr.is_terminal();
    let mut out = stderr.lock();

    if use_color {
        // Bold green label, right-aligned to 12 chars
        let _ = write!(out, "\x1b[1;32m{label:>12}\x1b[0m {message}\n");
    } else {
        let _ = write!(out, "{label:>12} {message}\n");
    }
}
