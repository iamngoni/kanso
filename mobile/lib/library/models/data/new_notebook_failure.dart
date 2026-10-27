// Kanso — new_notebook_failure.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why a new notebook was not made.
enum NewNotebookFailure {
  /// The name was empty, or the library would not take it.
  invalidName,

  /// Anything else.
  failed,
}
