// Kanso: import_labels.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/import_problem.dart';

/// The words of importing.
extension ImportLabels on AppLocalizations {
  /// Why a step of importing did not happen.
  String importProblemText(ImportProblem problem) {
    return switch (problem) {
      ImportProblemWords(:final message) => message,
      ImportProblemPickFailed() => importProblemPickFailed,
      ImportProblemNotAnExport() => importProblemNotAnExport,
      ImportProblemUnexpected() => importProblemUnexpected,
    };
  }
}
