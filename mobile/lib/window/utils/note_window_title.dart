// Kanso — note_window_title.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The name the window list and the app switcher show for a note's window.
///
/// A locked note is called "Locked note" whatever its title: the title of a
/// locked note is not shown in a window title or a scene title, which the
/// system keeps on screen and in its snapshots.
String noteWindowTitle({
  required String title,
  required bool locked,
  required String untitled,
  required String lockedTitle,
}) {
  if (locked) {
    return lockedTitle;
  }
  final name = title.trim();
  return name.isEmpty ? untitled : name;
}
