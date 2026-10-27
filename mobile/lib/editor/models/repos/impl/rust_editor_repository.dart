// Kanso — rust_editor_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/editor.dart';
import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/cancelable_stream.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/editor_repository.dart';

/// [EditorRepository] over the Rust core's library.
@LazySingleton(as: EditorRepository)
class RustEditorRepository implements EditorRepository {
  RustEditorRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, EditorDocument>> open(String noteId) =>
      attempt<EditorDocument>(
        () async => RustEditorDocument(await _library.edit(id: noteId)),
      );
}

/// [EditorDocument] over the Rust core's editor.
class RustEditorDocument implements EditorDocument {
  RustEditorDocument(this._editor);

  final KansoEditor _editor;

  @override
  Future<Either<ApplicationError, EditorSnapshot>> snapshot() =>
      attempt(_editor.snapshot);

  @override
  Either<ApplicationError, EditOutcome> apply(TextChange change) =>
      attemptSync(() => _editor.apply(change: change));

  @override
  Either<ApplicationError, EditorContext> select(
    Range16 selection, {
    required bool composing,
  }) => attemptSync(
    () => _editor.select(selection: selection, composing: composing),
  );

  @override
  Either<ApplicationError, EditOutcome?> command(EditCommand command) =>
      attemptSync(() => _editor.command(command: command));

  @override
  Either<ApplicationError, EditOutcome?> undo() => attemptSync(_editor.undo);

  @override
  Either<ApplicationError, EditOutcome?> redo() => attemptSync(_editor.redo);

  @override
  Either<ApplicationError, Unit> setTitle(String title) => attemptSync(() {
    _editor.setTitle(title: title);
    return unit;
  });

  @override
  Future<Either<ApplicationError, RunBatch>> styledRuns({
    required int start,
    required int end,
    required String themeId,
  }) => attempt(
    () => _editor.styledRuns(start: start, end: end, themeId: themeId),
  );

  @override
  Future<Either<ApplicationError, Completion?>> completion() =>
      attempt(_editor.completion);

  @override
  Future<Either<ApplicationError, LinkTarget?>> linkAt(int offset) =>
      attempt(() => _editor.linkAt(offset: offset));

  @override
  Future<Either<ApplicationError, List<Property>>> setProperty({
    required String key,
    required String value,
    int? index,
  }) => attempt(
    () => _editor.setProperty(index: index, key: key, value: value),
  );

  @override
  Future<Either<ApplicationError, List<Property>>> removeProperty(int index) =>
      attempt(() => _editor.removeProperty(index: index));

  @override
  Future<Either<ApplicationError, SaveState>> flush() => attempt(_editor.flush);

  @override
  Future<Either<ApplicationError, SaveState>> keepMine() =>
      attempt(_editor.keepMine);

  @override
  Future<Either<ApplicationError, EditorSnapshot>> useTheirs() =>
      attempt(_editor.useTheirs);

  @override
  Future<Either<ApplicationError, SaveState>> close() => attempt(_editor.close);

  @override
  Stream<EditorEvent> events() => cancelableStream(_editor.events());
}
