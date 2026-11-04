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
use std::sync::{Arc, mpsc};

use egui::collapsing_header::CollapsingState;

/// A folder's entry in the tree.
#[derive(Clone)]
struct Item {
    path: PathBuf,
    name: String,
    folder: bool,
}

/// A folder's entry as a [`Lister`] reports it.
pub struct Entry {
    pub name: String,
    pub folder: bool,
}

/// Lists a folder somewhere other than this machine's disk (ADR-0030): its
/// entries in any order. The tree skips hidden ones, drops files unless
/// [`FolderTree::files`] is on, and sorts. It runs on a worker thread.
pub type Lister = Arc<dyn Fn(&Path) -> Result<Vec<Entry>, String> + Send + Sync>;

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
    /// Where folders are listed from; this machine's disk if none.
    lister: Option<Lister>,
    /// What the filter box holds: only names containing it are shown.
    filter: String,
    /// The filter's match the keyboard is on, from the top.
    cursor: usize,
    /// Enter was pressed in the filter box: open the match under the cursor.
    open_cursor: bool,
    /// The cursor moved: scroll it into view.
    cursor_moved: bool,
}

/// Folders listed at once while a filter looks through the whole tree.
const SCAN_AT_ONCE: usize = 8;
/// Folders a filter doesn't look inside unless they're already listed: build
/// output and installed packages, which would bury what's wanted.
const SCAN_SKIPS: [&str; 2] = ["target", "node_modules"];
/// Most matches a filter lists.
const SHOWN_AT_MOST: usize = 500;
/// How deep a filter looks, so a link back up the tree can't loop.
const SCAN_DEPTH: usize = 16;

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
            lister: None,
            filter: String::new(),
            cursor: 0,
            open_cursor: false,
            cursor_moved: false,
        }
    }

    /// Lists each folder's files after its subfolders, for a project or
    /// notes sidebar; `show` then returns clicked files too.
    pub fn files(mut self, on: bool) -> Self {
        self.files = on;
        self
    }

    /// Lists folders with `lister` instead of reading this machine's disk,
    /// for a project that lives on another one (ADR-0030).
    pub fn lister(mut self, lister: Lister) -> Self {
        self.lister = Some(lister);
        self
    }

    /// A box for the filter, to go above [`Self::show`]: as it's typed in, the
    /// tree gives way to a flat list of the files (and folders) whose names contain it,
    /// however they're written (`how it works` finds `HOW-IT-WORKS.md`), each with its folder. Folders not yet
    /// listed are listed to look inside them. Escape empties it. Whether the
    /// text changed.
    pub fn filter_box(&mut self, ui: &mut egui::Ui) -> bool {
        use egui::{Key, Modifiers};
        let id = egui::Id::new("folder_tree_filter");
        let mut changed = false;
        // Keys that move through the matches, taken before the box sees them.
        if ui.memory(|m| m.has_focus(id)) && !self.filter.trim().is_empty() {
            let step = ui.input_mut(|i| {
                let down = i.consume_key(Modifiers::CTRL, Key::N) || i.consume_key(Modifiers::NONE, Key::ArrowDown);
                let up = i.consume_key(Modifiers::CTRL, Key::P) || i.consume_key(Modifiers::NONE, Key::ArrowUp);
                i64::from(down) - i64::from(up)
            });
            if step != 0 {
                self.cursor = self.cursor.saturating_add_signed(step as isize);
                self.cursor_moved = true;
            }
        }
        let edit = egui::TextEdit::singleline(&mut self.filter)
            .id(id)
            .hint_text("Filter files")
            .desired_width(f32::INFINITY);
        let resp = ui.add(edit);
        if resp.has_focus() && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.filter.clear();
            changed = true;
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            self.open_cursor = true;
            // Back in the box, to go on typing.
            resp.request_focus();
        }
        if resp.changed() || changed {
            self.cursor = 0;
            changed = true;
        }
        changed
    }

    /// Sets the filter, as typing in [`Self::filter_box`] does.
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
    }

    pub fn filter(&self) -> &str {
        &self.filter
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
        let lister = self.lister.clone();
        std::thread::spawn(move || {
            let result = match lister {
                Some(lister) => lister(&path).map(|entries| arrange(&path, entries, files)),
                None => list(&path, files),
            };
            let _ = tx.send(Listing { path, result });
            ctx.request_repaint();
        });
    }

    /// Draws the tree; returns a folder, or with [`Self::files`] a file, the
    /// user clicked. `current` is shown selected.
    pub fn show(&mut self, ui: &mut egui::Ui, current: &Path) -> Option<PathBuf> {
        // Hovering a row must not resize it (see `skin::steady_controls`).
        ui.scope(|ui| {
            crate::skin::steady_controls(ui);
            self.show_rows(ui, current)
        })
        .inner
    }

    fn show_rows(&mut self, ui: &mut egui::Ui, current: &Path) -> Option<PathBuf> {
        if !self.filter.trim().is_empty() {
            return self.show_filtered(ui, current);
        }
        let mut clicked = None;
        let root = self.root.clone();
        let label = self.root_label.clone();
        self.node(ui, &root, &label, current, true, &mut clicked);
        clicked
    }

    /// The filter's matches as a flat list: each name with the folder it's
    /// in, dimmed, beside it. Looks in each folder below the root, listing
    /// those not listed yet a few at a time.
    fn show_filtered(&mut self, ui: &mut egui::Ui, _current: &Path) -> Option<PathBuf> {
        let query = self.filter.clone();
        let (found, pending) = self.scan(&query, ui.ctx());
        let mut clicked = None;
        let shown = found.len().min(SHOWN_AT_MOST);
        self.cursor = self.cursor.min(shown.saturating_sub(1));
        if std::mem::take(&mut self.open_cursor) {
            clicked = found.get(self.cursor).map(|item| item.path.clone());
        }
        for (n, item) in found.iter().take(SHOWN_AT_MOST).enumerate() {
            let dir = item.path.parent().and_then(|d| d.strip_prefix(&self.root).ok());
            let icon = if item.folder { "\u{1F4C1}" } else { "\u{1F4C4}" };
            // The name, and under it the folder it's in, dimmed.
            let resp = ui
                .vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let resp = ui.selectable_label(n == self.cursor, format!("{icon} {}", item.name));
                    if let Some(dir) = dir.filter(|d| !d.as_os_str().is_empty()) {
                        let dir = format!("      {}", dir.display());
                        ui.label(egui::RichText::new(dir).weak().small());
                    }
                    resp
                })
                .inner
                .on_hover_text(item.path.display().to_string());
            if resp.clicked() {
                clicked = Some(item.path.clone());
                self.cursor = n;
            }
            if n == self.cursor && std::mem::take(&mut self.cursor_moved) {
                resp.scroll_to_me(None);
            }
        }
        if found.len() > SHOWN_AT_MOST {
            let more = format!("{} more: type more to narrow it", found.len() - SHOWN_AT_MOST);
            ui.label(egui::RichText::new(more).weak().small());
        }
        if pending {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new("Looking…").weak().small());
            });
        } else if found.is_empty() {
            ui.label(egui::RichText::new("Nothing matches").weak().small());
        }
        clicked
    }

    /// The entries below the root whose names match `query` ([`fuzzy`]), best
    /// first, from what's listed; whether folders are still to be listed.
    /// Asks for some of those.
    fn scan(&mut self, query: &str, ctx: &egui::Context) -> (Vec<Item>, bool) {
        let mut busy = self
            .children
            .values()
            .filter(|c| matches!(c, Children::Loading))
            .count();
        let mut pending = busy > 0;
        let mut found: Vec<(Rank, Item)> = Vec::new();
        let mut stack = vec![(self.root.clone(), 0)];
        while let Some((dir, depth)) = stack.pop() {
            match self.children.get(&dir) {
                None => {
                    pending = true;
                    if busy < SCAN_AT_ONCE {
                        self.request(&dir, ctx);
                        busy += 1;
                    }
                }
                Some(Children::Loaded(list)) => {
                    for item in list {
                        if let Some(rank) = fuzzy(&item.name, query) {
                            found.push((rank, item.clone()));
                        }
                        let skipped = SCAN_SKIPS.contains(&item.name.as_str())
                            && !self.children.contains_key(&item.path);
                        if item.folder && depth < SCAN_DEPTH && !skipped {
                            stack.push((item.path.clone(), depth + 1));
                        }
                    }
                }
                Some(Children::Loading | Children::Failed(_)) => {}
            }
        }
        found.sort_by_key(|(rank, item)| (*rank, item.name.len(), item.name.to_lowercase(), item.path.clone()));
        (found.into_iter().map(|(_, item)| item).collect(), pending)
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

/// How well a name matches a filter: lower is better.
type Rank = u8;

/// Whether `name` matches `query`, and how well. Case, and whether words are
/// parted by a space, dash, dot or underscore, don't matter, so `how it
/// works` finds `HOW-IT-WORKS.md`. Best: the query's letters run together in
/// the name; then each word of the query somewhere in it, in any order; then
/// the query's letters in order with others between (three or more).
fn fuzzy(name: &str, query: &str) -> Option<Rank> {
    let letters = |s: &str| -> Vec<char> {
        s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
    };
    let (want, have) = (letters(query), letters(name));
    if want.is_empty() {
        return None;
    }
    if have.windows(want.len()).any(|w| w == want.as_slice()) {
        return Some(0);
    }
    let lower = name.to_lowercase();
    if query.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).all(|w| lower.contains(&w.to_lowercase())) {
        return Some(1);
    }
    let mut at = have.iter();
    (want.len() >= 3 && want.iter().all(|c| at.any(|h| h == c))).then_some(2)
}

/// Subfolders of `dir`, then with `files` its files, each sorted
/// case-insensitively, hidden ones skipped.
fn list(dir: &Path, files: bool) -> Result<Vec<Item>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        _ => e.to_string(),
    })?;
    let entries = entries
        .flatten()
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            // d_type avoids a stat per entry; only symlinks need following.
            let folder = match entry.file_type() {
                Ok(ft) if ft.is_symlink() => entry.path().is_dir(),
                Ok(ft) => ft.is_dir(),
                Err(_) => false,
            };
            Entry { name, folder }
        })
        .collect();
    Ok(arrange(dir, entries, files))
}

/// What a tree shows of `entries`: no hidden ones, files only with `files`,
/// folders first, each sorted case-insensitively.
fn arrange(dir: &Path, entries: Vec<Entry>, files: bool) -> Vec<Item> {
    let mut out: Vec<Item> = entries
        .into_iter()
        .filter(|entry| !entry.name.starts_with('.') && (entry.folder || files))
        .map(|Entry { name, folder }| Item {
            path: dir.join(&name),
            name,
            folder,
        })
        .collect();
    out.sort_by_key(|item| (!item.folder, item.name.to_lowercase()));
    out
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
    fn another_lister_s_entries_are_arranged_the_same() {
        let entries = [
            Entry {
                name: "b.rs".into(),
                folder: false,
            },
            Entry {
                name: ".git".into(),
                folder: true,
            },
            Entry {
                name: "src".into(),
                folder: true,
            },
            Entry {
                name: "A.rs".into(),
                folder: false,
            },
        ];
        let names = |files| -> Vec<String> {
            let entries = entries
                .iter()
                .map(|e| Entry {
                    name: e.name.clone(),
                    folder: e.folder,
                })
                .collect();
            arrange(Path::new("/remote/p"), entries, files)
                .into_iter()
                .map(|item| item.path.to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(names(false), ["/remote/p/src"]);
        assert_eq!(
            names(true),
            ["/remote/p/src", "/remote/p/A.rs", "/remote/p/b.rs"]
        );
    }

    #[test]
    fn a_filter_lists_matches_flat_by_name() {
        let dir = std::env::temp_dir().join(format!("patina-tree-filter-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/deep")).unwrap();
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        for f in ["src/deep/Needle.rs", "src/other.rs", "docs/a.md", "docs/needle.md"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        let mut tree = FolderTree::new(dir.clone(), "p".into()).files(true);
        let ctx = egui::Context::default();
        // Listing is on threads: scan until nothing is pending.
        let mut found = (Vec::new(), true);
        for _ in 0..200 {
            tree.poll();
            found = tree.scan("needle", &ctx);
            if !found.1 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(!found.1, "scan finished");
        let names: Vec<String> = found
            .0
            .iter()
            .map(|i| i.path.strip_prefix(&dir).unwrap().to_string_lossy().into_owned())
            .collect();
        // Only the matches, by name, whatever folder they're in.
        assert_eq!(names, ["docs/needle.md", "src/deep/Needle.rs"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_filter_is_fuzzy() {
        // Case and separators don't matter.
        assert_eq!(fuzzy("HOW-IT-WORKS.md", "how it works"), Some(0));
        assert_eq!(fuzzy("HOW-IT-WORKS.md", "howitworks"), Some(0));
        assert_eq!(fuzzy("how_it_works.md", "How-It"), Some(0));
        // Words in any order, then letters in order.
        assert_eq!(fuzzy("HOW-IT-WORKS.md", "works how"), Some(1));
        assert_eq!(fuzzy("HOW-IT-WORKS.md", "hwrk"), Some(2));
        assert_eq!(fuzzy("HOW-IT-WORKS.md", "hw"), None);
        assert_eq!(fuzzy("README.md", "how it works"), None);
        assert_eq!(fuzzy("README.md", "  "), None);
    }

    /// One frame of the tree's filter box and list, with `events` pressed.
    fn frame(ctx: &egui::Context, tree: &mut FolderTree, events: Vec<egui::Event>) -> Option<PathBuf> {
        let input = egui::RawInput { events, ..Default::default() };
        let mut opened = None;
        let _ = ctx.run_ui(input, |ui| {
            tree.poll();
            tree.filter_box(ui);
            opened = tree.show(ui, Path::new(""));
        });
        opened
    }

    fn press(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key { key, physical_key: Some(key), pressed: true, repeat: false, modifiers }
    }

    #[test]
    fn ctrl_n_and_p_move_through_matches_and_enter_opens() {
        let dir = std::env::temp_dir().join(format!("patina-tree-keys-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["a.rs", "b.rs", "c.rs"] {
            std::fs::write(dir.join(f), b"").unwrap();
        }
        let mut tree = FolderTree::new(dir.clone(), "p".into()).files(true);
        tree.set_filter(".rs");
        let ctx = egui::Context::default();
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("folder_tree_filter")));
        let none = egui::Modifiers::NONE;
        // Let the folder be listed.
        for _ in 0..100 {
            frame(&ctx, &mut tree, vec![]);
            if !tree.scan(".rs", &ctx).1 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        frame(&ctx, &mut tree, vec![]);
        frame(&ctx, &mut tree, vec![press(egui::Key::N, egui::Modifiers::CTRL)]);
        frame(&ctx, &mut tree, vec![press(egui::Key::N, egui::Modifiers::CTRL)]);
        frame(&ctx, &mut tree, vec![press(egui::Key::P, egui::Modifiers::CTRL)]);
        assert_eq!(tree.cursor, 1);
        frame(&ctx, &mut tree, vec![press(egui::Key::ArrowDown, none)]);
        frame(&ctx, &mut tree, vec![press(egui::Key::ArrowDown, none)]);
        assert_eq!(tree.cursor, 2, "stops at the last match");
        frame(&ctx, &mut tree, vec![press(egui::Key::ArrowUp, none)]);
        let opened = frame(&ctx, &mut tree, vec![press(egui::Key::Enter, none)]);
        assert_eq!(opened, Some(dir.join("b.rs")));
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
