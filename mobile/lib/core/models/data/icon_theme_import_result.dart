// Kanso — icon_theme_import_result.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'icon_theme_import_result.freezed.dart';

/// The icon themes an import added, and what the importer made of them.
@freezed
abstract class IconThemeImportResult with _$IconThemeImportResult {
  const factory IconThemeImportResult({
    /// The imported icon themes' ids.
    required List<String> ids,

    /// The imported icon themes' names, in the same order.
    required List<String> names,

    /// A sentence on what was imported (how many icons, which were left
    /// out); empty when there is nothing to add.
    required String summary,
  }) = _IconThemeImportResult;
}
