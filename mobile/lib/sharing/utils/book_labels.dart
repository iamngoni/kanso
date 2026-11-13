// Kanso — book_labels.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../../core/bridge/api/types.dart';
import '../../core/utils/time_words.dart';
import '../../gen/l10n/app_localizations.dart';

/// A role in a shared notebook, as the desktop names it: "Can edit".
String bookRoleLabel(AppLocalizations l10n, BookRole role) =>
    l10n.bookRoleName(role.name);

/// The line under a shared notebook, as the desktop's: "Up to date",
/// "2 changes waiting", "Offline · 4 changes waiting".
String bookStatusLabel(AppLocalizations l10n, BookSyncStatus status) {
  final waiting = status.waiting;
  return switch ((status.offline, waiting)) {
    (true, 0) => l10n.bookStatusOffline,
    (true, _) => l10n.bookStatusOfflineWaiting(waiting),
    (false, 0) => l10n.bookStatusUpToDate,
    (false, _) => l10n.bookStatusWaiting(waiting),
  };
}

/// "Edited by Dan · 2 min ago", or "Edited by you · just now".
String editedByLabel(
  AppLocalizations l10n,
  NoteEditRow edit, {
  required DateTime now,
}) {
  final when = editedLabel(l10n, edit.atMs, now: now);
  return edit.me
      ? l10n.noteEditedByYou(when)
      : l10n.noteEditedBy(edit.name, when);
}

/// The first letter or digit of a name, upper-cased, for its avatar; "?"
/// for a name with none.
String avatarInitial(String name) {
  for (final rune in name.runes) {
    final letter = String.fromCharCode(rune);
    if (RegExp(r'[\p{L}\p{N}]', unicode: true).hasMatch(letter)) {
      return letter.toUpperCase();
    }
  }
  return '?';
}

/// The colours a person's avatar is drawn in, the same for a name on every
/// device: the eight hues the editor gives other people's cursors, as each
/// shows on a light page and a dark one (`kanso-engine`'s `CURSOR_COLOURS`).
const List<(Color, Color)> avatarColours = <(Color, Color)>[
  (Color(0xFF2F6AAE), Color(0xFF7FB0EC)),
  (Color(0xFF2D7A4D), Color(0xFF76C59C)),
  (Color(0xFFA6531A), Color(0xFFEDA160)),
  (Color(0xFF7550AE), Color(0xFFB89CEE)),
  (Color(0xFFB03A3A), Color(0xFFEE8484)),
  (Color(0xFF1E7474), Color(0xFF6CCBCB)),
  (Color(0xFF9E3B80), Color(0xFFE68BC6)),
  (Color(0xFF666C1E), Color(0xFFC6C672)),
];

/// Which of [avatarColours] a name gets: a stable hash of it.
int avatarTone(String name) {
  var hash = 0x811c9dc5;
  for (final unit in name.codeUnits) {
    hash = ((hash ^ unit) * 0x01000193) & 0xffffffff;
  }
  return hash % avatarColours.length;
}
