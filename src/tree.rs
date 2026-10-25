//! A sidebar folder tree: a collapsible tree under a root folder whose
//! subfolders are listed on worker threads the first time a folder is
//! expanded, so a slow or hung network share never blocks the UI. Moved from
//! UnAmp (its ADR-0014).
//!
//! Call [`FolderTree::poll`] once a frame and [`FolderTree::show`] where the
//! tree goes; `show` returns the folder the user clicked. With
//! [`FolderTree::files`] on, each folder lists its files after its
//! subfolders, and `show` returns a clicked file too (ADR-0029).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use egui::collapsing_header::CollapsingState;

/// A folder's entry in the tree.
#[derive(Clone)]
struct Item {
    path: PathBuf,
    name: String,
    folder: bool,
}

enum Children {
    Loading,
    Loaded(Vec<Item>),
    Failed(String),
}

struct Listing {
    path: PathBuf,
    result: Result<Vec<Item>, String>,
}

pub struct FolderTree {
    root: PathBuf,
    root_label: String,
    children: HashMap<PathBuf, Children>,
    tx: mpsc::Sender<Listing>,
    rx: mpsc::Receiver<Listing>,
    /// Folder whose ancestors should be opened (and which should be
    /// scrolled into view) once the tree has loaded far enough to show it.
    reveal: Option<PathBuf>,
    /// Lists files as well as folders.
    files: bool,
}

impl FolderTree {
    pub fn new(root: PathBuf, root_label: String) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            root,
            root_label,
            children: HashMap::new(),
            tx,
            rx,
            reveal: None,
            files: false,
        }
    }

    /// Lists each folder's files after its subfolders, for a project or
    /// notes sidebar; `show` then returns clicked files too.
    pub fn files(mut self, on: bool) -> Self {
        self.files = on;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn root_label(&self) -> &str {
        &self.root_label
    }

    /// Re-roots the tree. Listings already loaded are kept.
    pub fn set_root(&mut self, root: PathBuf, label: String) {
        self.root = root;
        self.root_label = label;
    }

    /// Opens `path` and the folders leading to it, and scrolls it into view.
    pub fn reveal(&mut self, path: &Path) {
        self.reveal = path.starts_with(&self.root).then(|| path.to_path_buf());
    }

    /// Forgets `path`'s listing so it's read again next time it's shown.
    pub fn refresh(&mut self, path: &Path) {
        if !matches!(self.children.get(path), Some(Children::Loading)) {
            self.children.remove(path);
        }
    }

    pub fn poll(&mut self) {
        while let Ok(Listing { path, result }) = self.rx.try_recv() {
            let children = match result {
                Ok(list) => Children::Loaded(list),
                Err(e) => Children::Failed(e),
            };
            self.children.insert(path, children);
        }
    }

    fn request(&mut self, path: &Path, ctx: &egui::Context) {
        self.children.insert(path.to_path_buf(), Children::Loading);
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let path = path.to_path_buf();
        let files = self.files;
        std::thread::spawn(move || {
            let result = list(&path, files);
            let _ = tx.send(Listing { path, result });
            ctx.request_repaint();
        });
    }

    /// Draws the tree; returns a folder, or with [`Self::files`] a file, the
    /// user clicked. `current` is shown selected.
    pub fn show(&mut self, ui: &mut egui::Ui, current: &Path) -> Option<PathBuf> {
        let mut clicked = None;
        let root = self.root.clone();
        let label = self.root_label.clone();
        self.node(ui, &root, &label, current, true, &mut clicked);
        clicked
    }

    fn node(
        &mut self,
        ui: &mut egui::Ui,
        path: &Path,
        name: &str,
        current: &Path,
        is_root: bool,
        clicked: &mut Option<PathBuf>,
    ) {
        let id = ui.make_persistent_id(("folder_tree", path));
        let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, is_root);
        // Opens the folders leading to the revealed one, and that folder
        // itself, so clicking a folder both selects and expands it.
        let revealing = self.reveal.as_deref().is_some_and(|r| r.starts_with(path));
        if revealing && !state.is_open() {
            state.set_open(true);
        }
        let is_current = path == current;
        let icon = if is_current { "\u{1F4C2}" } else { "\u{1F4C1}" };
        let label = format!("{icon} {name}");

        let header = |ui: &mut egui::Ui, clicked: &mut Option<PathBuf>| {
            let resp = ui
                .selectable_label(is_current, &label)
                .on_hover_text(path.display().to_string());
            if resp.clicked() {
                *clicked = Some(path.to_path_buf());
            }
            resp
        };

        // A folder known to have nothing in it gets no expander.
        if matches!(self.children.get(path), Some(Children::Loaded(c)) if c.is_empty()) {
            let resp = ui
                .horizontal(|ui| {
                    ui.add_space(ui.spacing().icon_width + ui.spacing().icon_spacing);
                    header(ui, clicked)
                })
                .inner;
            self.finish_reveal(path, &resp);
            return;
        }

        let header_resp = state.show_header(ui, |ui| header(ui, clicked));
        let (_, header_out, _) = header_resp.body(|ui| match self.children.get(path) {
            None => {
                self.request(path, ui.ctx());
                ui.spinner();
            }
            Some(Children::Loading) => {
                ui.spinner();
            }
            Some(Children::Failed(e)) => {
                ui.label(egui::RichText::new(e.as_str()).weak().small());
            }
            Some(Children::Loaded(list)) => {
                for item in list.clone() {
                    if item.folder {
                        self.node(ui, &item.path, &item.name, current, false, clicked);
                    } else {
                        self.file(ui, &item, current, clicked);
                    }
                }
            }
        });
        self.finish_reveal(path, &header_out.inner);
    }

    /// A file: a leaf, lined up with the folders' names.
    fn file(
        &mut self,
        ui: &mut egui::Ui,
        item: &Item,
        current: &Path,
        clicked: &mut Option<PathBuf>,
    ) {
        let resp = ui
            .horizontal(|ui| {
                ui.add_space(ui.spacing().icon_width + ui.spacing().icon_spacing);
                ui.selectable_label(item.path == current, format!("\u{1F4C4} {}", item.name))
                    .on_hover_text(item.path.display().to_string())
            })
            .inner;
        if resp.clicked() {
            *clicked = Some(item.path.clone());
        }
        self.finish_reveal(&item.path, &resp);
    }

    fn finish_reveal(&mut self, path: &Path, resp: &egui::Response) {
        if self.reveal.as_deref() == Some(path) {
            resp.scroll_to_me(Some(egui::Align::Center));
            self.reveal = None;
        }
    }
}

/// Subfolders of `dir`, then with `files` its files, each sorted
/// case-insensitively, hidden ones skipped.
fn list(dir: &Path, files: bool) -> Result<Vec<Item>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        _ => e.to_string(),
    })?;
    let mut out: Vec<Item> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            let path = entry.path();
            // d_type avoids a stat per entry; only symlinks need following.
            let folder = match entry.file_type() {
                Ok(ft) if ft.is_symlink() => path.is_dir(),
                Ok(ft) => ft.is_dir(),
                Err(_) => false,
            };
            (folder || files).then_some(Item { path, name, folder })
        })
        .collect();
    out.sort_by_key(|item| (!item.folder, item.name.to_lowercase()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_visible_subfolders_sorted() {
        let dir = std::env::temp_dir().join(format!("patina-tree-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for d in ["beta", "Alpha", ".hidden", "gamma/deeper"] {
            std::fs::create_dir_all(dir.join(d)).unwrap();
        }
        std::fs::write(dir.join("song.mp3"), b"").unwrap();
        let names = |files| -> Vec<String> {
            list(&dir, files)
                .unwrap()
                .into_iter()
                .map(|item| item.name)
                .collect()
        };
        assert_eq!(names(false), ["Alpha", "beta", "gamma"]);
        // Files come after the folders, hidden ones still skipped.
        std::fs::write(dir.join("Apple.rs"), b"").unwrap();
        std::fs::write(dir.join(".env"), b"").unwrap();
        assert_eq!(
            names(true),
            ["Alpha", "beta", "gamma", "Apple.rs", "song.mp3"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unreadable_folder_is_an_error() {
        assert!(list(Path::new("/definitely/not/here"), true).is_err());
    }

    #[test]
    fn reveal_ignores_paths_outside_the_root() {
        let mut tree = FolderTree::new(PathBuf::from("/music"), "Music".into());
        tree.reveal(Path::new("/elsewhere/x"));
        assert!(tree.reveal.is_none());
        tree.reveal(Path::new("/music/a/b"));
        assert_eq!(tree.reveal.as_deref(), Some(Path::new("/music/a/b")));
    }
}
