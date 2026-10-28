// Kanso — fake_note_window_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/window/models/data/note_window_info.dart';
import 'package:kanso/window/models/data/note_window_request.dart';
import 'package:kanso/window/models/repos/abs/note_window_repository.dart';

/// A [NoteWindowRepository] with no windows behind it: it remembers what it
/// was asked for.
class FakeNoteWindowRepository implements NoteWindowRepository {
  FakeNoteWindowRepository({this.supported = true});

  @override
  final bool supported;

  /// The notes asked to be opened in a window, in order.
  final List<String> opened = <String>[];

  /// How many times this window was asked to close.
  int closed = 0;

  /// The names the window was given, in order.
  final List<String> titles = <String>[];

  /// What the window handed to the main window, in order.
  final List<NoteWindowRequest> handed = <NoteWindowRequest>[];

  final StreamController<NoteWindowRequest> incoming =
      StreamController<NoteWindowRequest>.broadcast();

  @override
  Future<Either<ApplicationError, Unit>> open(String noteId) async {
    opened.add(noteId);
    return right(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> closeThisWindow() async {
    closed++;
    return right(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setTitle(String title) async {
    titles.add(title);
    return right(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> hand(NoteWindowRequest request) async {
    handed.add(request);
    return right(unit);
  }

  /// The notes whose windows were asked to close, in order.
  final List<String> closedNotes = <String>[];

  /// What [openWindows] answers.
  List<NoteWindowInfo> windows = <NoteWindowInfo>[];

  @override
  Future<Either<ApplicationError, Unit>> closeWindowOf(String noteId) async {
    closedNotes.add(noteId);
    return right(unit);
  }

  @override
  Future<Either<ApplicationError, List<NoteWindowInfo>>> openWindows() async =>
      right(windows);

  @override
  Stream<NoteWindowRequest> requests() => incoming.stream;
}
