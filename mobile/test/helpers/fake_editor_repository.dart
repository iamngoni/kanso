// Kanso — fake_editor_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';

/// An [EditorRepository] that opens the document it was given, or fails with
/// the error it was given, and counts the openings.
class FakeEditorRepository implements EditorRepository {
  FakeEditorRepository({this.document, this.error});

  final EditorDocument? document;
  final ApplicationError? error;
  final List<String> opened = <String>[];

  @override
  Future<Either<ApplicationError, EditorDocument>> open(String noteId) async {
    opened.add(noteId);
    final failure = error;
    if (failure != null) {
      return Left<ApplicationError, EditorDocument>(failure);
    }
    return Right<ApplicationError, EditorDocument>(document!);
  }
}
