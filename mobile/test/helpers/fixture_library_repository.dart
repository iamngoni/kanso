// Kanso — fixture_library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';

import 'library_fixtures.dart';

/// A library of the mock content, for widget and golden tests: every call
/// answers from a field a test can change, and records what it was asked.
class FixtureLibraryRepository implements LibraryRepository {
  FixtureLibraryRepository() {
    for (final row in mockRows) {
      if (row.id == MockIds.syncDesign) {
        notes[row.id] = mockOpenNote;
      } else if (row.locked) {
        notes[row.id] = mockLockedNote;
      }
    }
  }

  List<NotebookRow> notebookRows = mockNotebooks;
  List<TagRow> tagRows = mockTags;
  LibraryOverview overviewValue = mockOverview;

  /// What `list` answers; a failure shows the list's error state.
  Either<ApplicationError, NoteList> listResult =
      Right<ApplicationError, NoteList>(mockNoteList);

  /// The notes `note` knows; others are not found.
  final Map<String, NoteView> notes = <String, NoteView>{};

  /// What `search` answers.
  Either<ApplicationError, List<Hit>> searchResult =
      Right<ApplicationError, List<Hit>>(mockHits);

  /// What `change` answers.
  Either<ApplicationError, Changed> changeResult =
      const Right<ApplicationError, Changed>(
        Changed(count: 1, undo: 1, message: 'Pinned 1 note.'),
      );

  /// What `undo` answers.
  Either<ApplicationError, int> undoResult = const Right<ApplicationError, int>(
    1,
  );

  /// What `tasks` answers; a failure shows the Tasks list's error state.
  Either<ApplicationError, TaskBoard> tasksResult =
      const Right<ApplicationError, TaskBoard>(mockTaskBoard);

  /// What `overdueTasks` answers.
  int overdueValue = mockOverdueTasks;

  /// What `setTask` answers.
  Either<ApplicationError, Ticked> setTaskResult =
      const Right<ApplicationError, Ticked>(
        Ticked.changed(rev: 'r2', undo: 7, message: 'Ticked off “Call Sam”'),
      );

  /// When set, `tasks` waits for it: tasks that are still loading.
  Completer<void>? tasksGate;

  /// The queries `tasks` was asked.
  final List<TaskListQuery> taskQueries = <TaskListQuery>[];

  /// The ticks `setTask` was asked for: note, line, text, checked.
  final List<(String, int, String, bool)> tasksSet =
      <(String, int, String, bool)>[];

  /// The boxes `toggleTaskAt` was asked to tick: note, index, rev.
  final List<(String, int, String)> boxesToggled = <(String, int, String)>[];

  /// The titles `createLinkedNote` was asked for, with the note linking.
  final List<(String, String)> linkedMade = <(String, String)>[];

  /// What `attachment` answers.
  Either<ApplicationError, AttachmentFile> attachmentResult =
      const Left<ApplicationError, AttachmentFile>(
        ApplicationError.notFound(message: 'No such file.'),
      );

  /// The tickets `undo` was asked to take back.
  final List<int> undone = <int>[];

  /// When set, `list` waits for it: a list that is still loading.
  Completer<void>? listGate;

  /// When set, `search` waits for it: a search that is still running.
  Completer<void>? searchGate;

  /// When set, `note` waits for it: a note that is still loading.
  Completer<void>? noteGate;

  /// When set, `note` fails with it for every note.
  ApplicationError? noteFailure;

  /// When set, `notebooks` waits for it: a library that is still loading.
  Completer<void>? notebooksGate;

  /// When set, `notebooks` fails with it: a library that cannot be read.
  ApplicationError? notebooksFailure;

  final List<ListQuery> listQueries = <ListQuery>[];
  final List<(List<String>, NoteChange)> changesMade =
      <(List<String>, NoteChange)>[];
  final List<(String, SearchScope)> searches = <(String, SearchScope)>[];
  final StreamController<List<String>> changeStream =
      StreamController<List<String>>.broadcast();

  Future<void> close() => changeStream.close();

  @override
  Future<Either<ApplicationError, NoteList>> list(ListQuery query) async {
    listQueries.add(query);
    await listGate?.future;
    return listResult;
  }

  @override
  Future<Either<ApplicationError, LibraryOverview>> overview() async =>
      Right<ApplicationError, LibraryOverview>(overviewValue);

  @override
  Future<Either<ApplicationError, int>> undo(int ticket) async {
    undone.add(ticket);
    return undoResult;
  }

  @override
  Future<Either<ApplicationError, Changed>> change(
    List<String> ids,
    NoteChange change,
  ) async {
    changesMade.add((ids, change));
    return changeResult;
  }

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) async {
    await noteGate?.future;
    final failure = noteFailure;
    if (failure != null) {
      return Left<ApplicationError, NoteView>(failure);
    }
    final note = notes[id];
    return note == null
        ? const Left<ApplicationError, NoteView>(
            ApplicationError.notFound(message: 'Not found.'),
          )
        : Right<ApplicationError, NoteView>(note);
  }

  @override
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks() async {
    await notebooksGate?.future;
    final failure = notebooksFailure;
    return failure == null
        ? Right<ApplicationError, List<NotebookRow>>(notebookRows)
        : Left<ApplicationError, List<NotebookRow>>(failure);
  }

  @override
  Future<Either<ApplicationError, List<TagRow>>> tags() async =>
      Right<ApplicationError, List<TagRow>>(tagRows);

  @override
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
    int limit = 50,
  }) async {
    searches.add((text, scope));
    await searchGate?.future;
    return searchResult;
  }

  @override
  Future<Either<ApplicationError, TaskBoard>> tasks(
    TaskListQuery query,
  ) async {
    taskQueries.add(query);
    await tasksGate?.future;
    return tasksResult;
  }

  @override
  Future<Either<ApplicationError, int>> overdueTasks() async =>
      Right<ApplicationError, int>(overdueValue);

  @override
  Future<Either<ApplicationError, Ticked>> setTask({
    required String noteId,
    required int line,
    required String text,
    required bool checked,
  }) async {
    tasksSet.add((noteId, line, text, checked));
    return setTaskResult;
  }

  @override
  Future<Either<ApplicationError, Ticked>> toggleTaskAt({
    required String noteId,
    required int index,
    required String baseRev,
  }) async {
    boxesToggled.add((noteId, index, baseRev));
    return setTaskResult;
  }

  @override
  Future<Either<ApplicationError, Saved>> createLinkedNote({
    required String title,
    required String fromNote,
  }) async {
    linkedMade.add((title, fromNote));
    return Right<ApplicationError, Saved>(
      Saved(id: 'note:linked-${linkedMade.length}', rev: '1'),
    );
  }

  @override
  Future<Either<ApplicationError, AttachmentFile>> attachment(
    String id,
  ) async => attachmentResult;

  @override
  Stream<List<String>> changes() => changeStream.stream;

  @override
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  }) => throw UnimplementedError();

  /// The notes `create` was asked to make.
  final List<NewNote> created = <NewNote>[];

  /// What `create` answers when it is set; a note with an id of its own
  /// otherwise.
  Either<ApplicationError, Saved>? createResult;

  @override
  Future<Either<ApplicationError, Saved>> create(NewNote note) async {
    created.add(note);
    return createResult ??
        Right<ApplicationError, Saved>(
          Saved(id: 'note:new-${created.length}', rev: '1-new'),
        );
  }

  @override
  Future<Either<ApplicationError, Unit>> trash(String id) =>
      throw UnimplementedError();

  /// The notebooks `createNotebook` was asked to make, with their parents.
  final List<(String, String?)> notebooksCreated = <(String, String?)>[];

  /// What `createNotebook` answers when it is set; the notebook, appended
  /// to the notebooks, otherwise.
  Either<ApplicationError, NotebookRow>? createNotebookResult;

  /// When set, `createNotebook` waits for it: a notebook still being made.
  Completer<void>? notebookGate;

  @override
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  }) async {
    notebooksCreated.add((name, parentId));
    await notebookGate?.future;
    final fixed = createNotebookResult;
    if (fixed != null) {
      return fixed;
    }
    final made = NotebookRow(
      id: 'book:made-${notebooksCreated.length}',
      name: name,
      parentId: parentId,
      count: 0,
      locked: false,
      depth: 0,
    );
    notebookRows = <NotebookRow>[...notebookRows, made];
    changeStream.add(<String>[made.id]);
    return Right<ApplicationError, NotebookRow>(made);
  }

  @override
  Future<Either<ApplicationError, List<Property>>> properties(String id) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  }) => throw UnimplementedError();
}
