// Kanso — tab_meta.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'tab_meta.freezed.dart';

/// Everything about a tab that survives the tab being closed and the note
/// changing elsewhere: its note, the title shown, and whether its text is
/// sealed (the desktop's `TabMeta`).
///
/// A locked note's title shows in its tab, with a lock beside it, as it does
/// in the list (so does the desktop's tab); its text is never part of a tab.
@freezed
abstract class TabMeta with _$TabMeta {
  const factory TabMeta({
    required String id,
    required String title,
    @Default(false) bool locked,
  }) = _TabMeta;

  const TabMeta._();

  /// Whether the note has a title to show.
  bool get hasTitle => title.trim().isNotEmpty;

  /// The title as the bar shows it: [untitled] for a blank one.
  String label(String untitled) => hasTitle ? title : untitled;
}
