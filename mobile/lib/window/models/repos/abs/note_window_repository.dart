// Kanso — note_window_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/note_window_info.dart';
import '../../data/note_window_request.dart';

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

  /// Names the window this engine draws, for the system's window list and
  /// switcher. The caller keeps a locked note's title out of it (see
  /// `noteWindowTitle`).
  Future<Either<ApplicationError, Unit>> setTitle(String title);

  /// Hands [request] to the main window, which is brought forward and does
  /// it: what a note's window can't do itself.
  Future<Either<ApplicationError, Unit>> hand(NoteWindowRequest request);

  /// Closes the window of note [noteId], when it has one, from whichever
  /// engine asks.
  Future<Either<ApplicationError, Unit>> closeWindowOf(String noteId);

  /// The note windows that are open now, with the names they have.
  Future<Either<ApplicationError, List<NoteWindowInfo>>> openWindows();

  /// The requests the note windows hand to the main window, heard in the
  /// main window's engine. Broadcast.
  Stream<NoteWindowRequest> requests();
}
