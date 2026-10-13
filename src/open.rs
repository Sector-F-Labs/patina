//! Opens files and folders in the desktop's own apps.
//!
//! egui's `Context::open_url` hands every URL to the web browser, `file://`
//! ones included, so "Open skins folder" through it shows the folder in
//! Firefox rather than the file manager. [`path`] asks the desktop instead:
//! `xdg-open` on Linux and the BSDs, `open` on macOS and `explorer` on
//! Windows. Web links still belong to `open_url`.

use std::io;
use std::path::Path;
use std::process::Command;

/// Opens `path` in the desktop's app for it: a folder in the file manager, an
/// image in the image viewer. Returns once the opener has started; it reports
/// an error only if the opener couldn't be run.
pub fn path(path: &Path) -> io::Result<()> {
    let mut child = opener().arg(path).spawn()?;
    // Reaped on a thread so it doesn't linger as a zombie until the app quits.
    std::thread::spawn(move || child.wait());
    Ok(())
}

fn opener() -> Command {
    if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(windows) {
        Command::new("explorer")
    } else {
        Command::new("xdg-open")
    }
}
