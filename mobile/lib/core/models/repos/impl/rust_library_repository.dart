// Kanso — rust_library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../services/external_changes.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/library_repository.dart';

/// [LibraryRepository] over the Rust core's library.
@LazySingleton(as: LibraryRepository)
class RustLibraryRepository implements LibraryRepository {
  /// [external] says when another process may have written; without it
  /// only the library's own changes are reported.
  RustLibraryRepository(this._library, [ExternalChanges? external])
    : _external = external ?? ExternalChanges();

  final KansoLibrary _library;
  final ExternalChanges _external;

  @override
  Future<Either<ApplicationError, int?>> defaultWordGoal() =>
      attempt(_library.defaultWordGoal);

  @override
  Future<Either<ApplicationError, int?>> setDefaultWordGoal(int? goal) =>
      attempt(() => _library.setDefaultWordGoal(goal: goal));

  @override
  Future<Either<ApplicationError, GoalSetting>> noteWordGoal(String id) =>
      attempt(() => _library.noteWordGoal(id: id));

  @override
  Future<Either<ApplicationError, Unit>> setNoteWordGoal(
    String id,
    GoalSetting setting,
  ) => attempt(() async {
    await _library.setNoteWordGoal(id: id, setting: setting);
    return unit;
  });

  @override
  Future<Either<ApplicationError, NoteList>> list(ListQuery query) =>
      attempt(() => _library.list(query: query));

  @override
  Future<Either<ApplicationError, LibraryOverview>> overview() =>
      attempt(_library.overview);

  @override
  Future<Either<ApplicationError, Changed>> change(
    List<String> ids,
    NoteChange change,
  ) => attempt(() => _library.change(ids: ids, change: change));

  @override
  Future<Either<ApplicationError, int>> undo(int ticket) =>
      attempt(() => _library.undo(ticket: ticket));

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) =>
      attempt(() => _library.note(id: id));

  @override
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  }) => attempt(() => _library.save(id: id, baseRev: baseRev, body: body));

  @override
  Future<Either<ApplicationError, Saved>> create(NewNote note) =>
      attempt(() => _library.create(note: note));

  @override
  Future<Either<ApplicationError, Unit>> trash(String id) => attempt(() async {
    await _library.trash(id: id);
    return unit;
  });

  @override
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks() =>
      attempt(_library.notebooks);

  @override
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  }) => attempt(() => _library.createNotebook(name: name, parentId: parentId));

  @override
  Future<Either<ApplicationError, List<TagRow>>> tags() =>
      attempt(_library.tags);

  @override
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
    int limit = 50,
  }) => attempt(() => _library.search(text: text, scope: scope, limit: limit));

  @override
  Future<Either<ApplicationError, List<Property>>> properties(String id) =>
      attempt(() => _library.properties(id: id));

  @override
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  }) => attempt(() => _library.queryView(source: source, selfId: selfId));

  @override
  Future<Either<ApplicationError, List<StatusChoice>>> statuses() async =>
      right(KansoLibrary.statuses());

  @override
  Future<Either<ApplicationError, int>> emptyTrash() =>
      attempt(_library.emptyTrash);

  @override
  Future<Either<ApplicationError, Saved>> duplicate(String id) =>
      attempt(() => _library.duplicate(id: id));

  @override
  Future<Either<ApplicationError, CopiedLink>> noteLink(
    String id, {
    required bool kanso,
  }) => attempt(() => _library.noteLink(id: id, kanso: kanso));

  @override
  Future<Either<ApplicationError, List<VisibleNotebook>>> visibleNotebooks(
    FoldState folds, {
    String? showing,
  }) => attempt(
    () => _library.visibleNotebooks(folds: folds, showing: showing),
  );

  @override
  Future<Either<ApplicationError, FoldState>> toggleFold(
    FoldState folds,
    String id, {
    String? showing,
  }) => attempt(
    () => _library.toggleFold(folds: folds, id: id, showing: showing),
  );

  @override
  Future<Either<ApplicationError, TagPlaces>> tagPlaces({
    required bool all,
    required List<String> openPaths,
    String? selected,
    String? showing,
  }) => attempt(
    () => _library.tagPlaces(
      all: all,
      selected: selected,
      openPaths: openPaths,
      showing: showing,
    ),
  );

  @override
  Future<Either<ApplicationError, Unit>> renameNotebook(
    String id,
    String name,
  ) => attempt(() async {
    await _library.renameNotebook(id: id, name: name);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> deleteNotebook(String id) =>
      attempt(() async {
        await _library.deleteNotebook(id: id);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> setNotebookIcon(
    String id,
    String? icon,
  ) => attempt(() async {
    await _library.setNotebookIcon(id: id, icon: icon);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> setNotebookColor(
    String id,
    String? color,
  ) => attempt(() async {
    await _library.setNotebookColor(id: id, color: color);
    return unit;
  });

  @override
  Future<Either<ApplicationError, TagRow>> createTag(String name) =>
      attempt(() => _library.createTag(name: name));

  @override
  Future<Either<ApplicationError, Unit>> renameTag(String id, String name) =>
      attempt(() async {
        await _library.renameTag(id: id, name: name);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> deleteTag(String id) =>
      attempt(() async {
        await _library.deleteTag(id: id);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> setTagColor(
    String id,
    String color,
  ) => attempt(() async {
    await _library.setTagColor(id: id, color: color);
    return unit;
  });

  @override
  Future<Either<ApplicationError, List<IconChoice>>> notebookIcons(
    String query,
  ) async => right(KansoLibrary.notebookIcons(query: query));

  @override
  Future<Either<ApplicationError, List<ColorChoice>>> notebookColors() async =>
      right(KansoLibrary.notebookColors());

  @override
  Future<Either<ApplicationError, List<ColorChoice>>> tagColors() async =>
      right(KansoLibrary.tagColors());

  @override
  Stream<List<String>> changes() {
    // What the library reports, and what another process may have written.
    final own = cancelableStream(_library.changes());
    late StreamController<List<String>> merged;
    StreamSubscription<List<String>>? ownSubscription;
    StreamSubscription<List<String>>? externalSubscription;
    merged = StreamController<List<String>>(
      onListen: () {
        ownSubscription = own.listen(
          merged.add,
          onError: merged.addError,
          onDone: merged.close,
        );
        externalSubscription = _external.stream.listen(merged.add);
      },
      onCancel: () async {
        await externalSubscription?.cancel();
        // Not awaited: the library's own stream ends at its next event.
        unawaited(ownSubscription?.cancel());
      },
    );
    return merged.stream;
  }

  @override
  Future<Either<ApplicationError, TaskBoard>> tasks(TaskListQuery query) =>
      attempt(() => _library.tasks(query: query));

  @override
  Future<Either<ApplicationError, int>> overdueTasks() =>
      attempt(_library.overdueTasks);

  @override
  Future<Either<ApplicationError, Ticked>> setTask({
    required String noteId,
    required int line,
    required String text,
    required bool checked,
  }) => attempt(
    () => _library.setTask(
      noteId: noteId,
      line: line,
      text: text,
      checked: checked,
    ),
  );

  @override
  Future<Either<ApplicationError, Ticked>> toggleTaskAt({
    required String noteId,
    required int index,
    required String baseRev,
  }) => attempt(
    () => _library.toggleTaskAt(noteId: noteId, index: index, baseRev: baseRev),
  );

  @override
  Future<Either<ApplicationError, Saved>> createLinkedNote({
    required String title,
    required String fromNote,
  }) => attempt(
    () => _library.createLinkedNote(title: title, fromNote: fromNote),
  );

  @override
  Future<Either<ApplicationError, AttachmentFile>> attachment(String id) =>
      attempt(() => _library.attachment(id: id));

  @override
  String? parseColor(String text) => KansoLibrary.parseColor(text: text);
}
