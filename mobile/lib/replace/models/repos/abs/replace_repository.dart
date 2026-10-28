// Kanso — replace_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// Replace in all notes: what the Rust core finds and changes.
///
/// A locked note is never read and never changed, closed or open: it is
/// not in a preview, and a plan that names one leaves it alone.
abstract interface class ReplaceRepository {
  /// Every match of what [options] look for, note by note, the most
  /// recently changed first. A pattern that cannot be used is the preview's
  /// `error`; no text to find previews nothing.
  Future<Either<ApplicationError, ReplacePreview>> preview(
    ReplaceOptions options,
  );

  /// Puts [replacement] in the matches of the [planned] notes in one write
  /// (`$1` and `${name}` stand for a regular expression's groups). A note
  /// changed since the preview, gone or locked is left alone and counted.
  Future<Either<ApplicationError, ReplaceOutcome>> apply({
    required ReplaceOptions options,
    required String replacement,
    required List<PlannedNote> planned,
  });

  /// Puts back what the replace under [batch] changed, except in notes
  /// edited since. A batch is undone once.
  Future<Either<ApplicationError, ReplaceUndone>> undo(String batch);
}
