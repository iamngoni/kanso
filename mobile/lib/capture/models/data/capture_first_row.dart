// Kanso — capture_first_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/configs/kanso_icons.dart';

part 'capture_first_row.freezed.dart';

/// The row a capture sheet starts with, above the camera's: a new note,
/// from the list's button, or an image link, from the editor's.
@freezed
abstract class CaptureFirstRow with _$CaptureFirstRow {
  const factory CaptureFirstRow({
    required KansoIcons icon,
    required String label,

    /// What choosing the row does, once the sheet has closed.
    required VoidCallback onTap,
  }) = _CaptureFirstRow;
}
