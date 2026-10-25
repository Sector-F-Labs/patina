# 0029. The folder tree can list files

Date: 2026-10-25

## Status

Accepted. The user asked for Programmer, a new IDE, to show its projects in Patina's tree. The
shape of the option below is my design.

## Context

`tree::FolderTree` came from UnAmp (ADR-0001), where a sidebar picks a folder of music, so it
lists only folders. Programmer opens a project folder in each tab, as Notebook does, and its
sidebar has to show the source files in it. Notebook has a tree of its own that lists its notes,
which reads the whole workspace on the window's thread when it opens. A large source tree (a
`target/` or `node_modules/` folder) would freeze the window doing that, which `FolderTree`'s lazy,
threaded listing already avoids.

We considered:

- **A second tree component for files.** It would copy the lazy listing, the reveal and the
  layout, and the two would drift (ADR-0001).
- **A filter callback choosing which entries to show.** More general, but no app needs more than
  "folders" or "folders and files" yet.

## Decision

- **`FolderTree::files(true)` lists each folder's files after its subfolders,** both sorted
  case-insensitively, hidden ones still skipped. Off by default, so UnAmp's tree is unchanged.
- **Files are leaves lined up with the folders' names.** `show` returns a clicked file as it
  returns a clicked folder, and `current` can be a file, shown selected. `reveal` opens the folders
  leading to a file and scrolls it into view.

## Consequences

- Programmer, and Notebook if it moves to it, get a sidebar that never blocks on a big folder.
- Every file is listed; there's no way yet to hide `target/` or what `.gitignore` names. An app
  that needs that is the trigger for a filter.
- The app tells files from folders by the path it gets back, which costs a stat on click.
