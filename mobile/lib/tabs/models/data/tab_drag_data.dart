// Kanso — tab_drag_data.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'tab_drag_data.freezed.dart';

/// A tab being dragged to a new place in the row.
@freezed
abstract class TabDragData with _$TabDragData {
  const factory TabDragData({
    /// Where in the row the tab was lifted from.
    required int from,

    /// The title it shows, for the lifted copy.
    required String title,
  }) = _TabDragData;
}
