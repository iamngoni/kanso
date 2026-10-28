// Kanso — open_tab.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/navigation_history.dart';
import 'tab_meta.dart';

part 'open_tab.freezed.dart';

/// One open note, with its own back and forward.
///
/// The desktop's tab also keeps where the reader was in the note; here that
/// is the note place repository's job (it keeps a place for every note), so a
/// tab holds none.
@freezed
abstract class OpenTab with _$OpenTab {
  const factory OpenTab({
    required TabMeta meta,
    required NavigationHistory history,
  }) = _OpenTab;

  const OpenTab._();

  /// A tab that has just opened [meta]: its first visit.
  factory OpenTab.opened(TabMeta meta) =>
      OpenTab(meta: meta, history: const NavigationHistory().visit(meta.id));
}
