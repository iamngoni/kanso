// Kanso — theme_import_outcome.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_import_outcome.freezed.dart';

/// How the last import of a theme or an icon theme ended, for the row that
/// says so.
@freezed
sealed class ThemeImportOutcome with _$ThemeImportOutcome {
  /// The file was imported: the [names] it added, and a [summary] sentence
  /// from the importer (empty when it has nothing to add).
  const factory ThemeImportOutcome.imported({
    required List<String> names,
    @Default('') String summary,
  }) = ThemeImportedOutcome;

  /// The file could not be imported or picked; [message] says why.
  const factory ThemeImportOutcome.failed({required String message}) =
      ThemeImportFailedOutcome;
}
