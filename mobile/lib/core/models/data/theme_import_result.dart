// Kanso — theme_import_result.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_import_result.freezed.dart';

/// The themes an import added to the themes folder.
@freezed
abstract class ThemeImportResult with _$ThemeImportResult {
  const factory ThemeImportResult({
    /// The imported themes' ids.
    required List<String> ids,

    /// The imported themes' names, in the same order.
    required List<String> names,
  }) = _ThemeImportResult;
}
