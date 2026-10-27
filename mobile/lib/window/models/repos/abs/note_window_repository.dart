// Kanso — note_window_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';

/// A note in a window of its own: an iPadOS scene, or an Android task window
/// beside the app (the desktop's `note_window.rs`).
abstract interface class NoteWindowRepository {
  /// Whether this device can show more than one window of the app: iPads
  /// and Android devices can, iPhones can't.
  bool get supported;

  /// Opens note [noteId] in a window of its own, or brings its window
  /// forward when it has one already.
  Future<Either<ApplicationError, Unit>> open(String noteId);

  /// Closes the window this engine draws, when it is a note's own window;
  /// the main window is never closed this way.
  Future<Either<ApplicationError, Unit>> closeThisWindow();
}
