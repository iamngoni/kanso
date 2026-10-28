// Kanso: import_problem.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'import_problem.freezed.dart';

/// Why a step of importing did not happen, as the screen says it.
@freezed
sealed class ImportProblem with _$ImportProblem {
  /// A sentence from the library, in the desktop's words: "Joplin export
  /// has no notes in it."
  const factory ImportProblem.words(String message) = ImportProblemWords;

  /// The system's picker, or the copy of what was chosen, failed.
  const factory ImportProblem.pickFailed() = ImportProblemPickFailed;

  /// The file is not an export Kanso reads.
  const factory ImportProblem.notAnExport() = ImportProblemNotAnExport;

  /// Anything else: nothing the person can do about it but try again.
  const factory ImportProblem.unexpected() = ImportProblemUnexpected;
}
