# Kanso 0.10.11 · What's new

## 1 · Links belong to Kanso

- Description: Notes, attachments and MCP now share one kanso:// address.
- Shows: A note's attachment link is now kanso://file:… and the MCP result returns kanso://note/…
- Covers: 8c0af86

## 2 · Choose your library

- Description: Choose a PouchDB library folder; the CLI requires its path.
- Shows: The Import chooser selects PouchDB library, the native Choose Library picker opens, the import dialog shows Copying the library… 0 of 1, and the CLI requires --library-dir.
- Covers: 8c0af86

## 3 · Linux starts clean

- Description: The GUI carries its XKB runtime; updates replace it whole.
- Shows: The installed support folder holds libxkbcommon-x11.so.0 and libxcb-xkb.so.1; the installer prints a missing library and its apt package, the .deb declares libxkbcommon-x11-0, and self-update swaps the folder atomically.
- Covers: da8ce6a

## 4 · Themes point home

- Description: Default theme and gallery metadata now point to the Kanso themes repository.
- Shows: Default Dark's source and its gallery/index.json entry both name github.com/iamngoni/kanso-themes.
- Covers: 8c0af86
