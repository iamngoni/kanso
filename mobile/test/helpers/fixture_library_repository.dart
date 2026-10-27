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
      depth: 0, hasChildren: false,
    );
    notebookRows = <NotebookRow>[...notebookRows, made];
    changeStream.add(<String>[made.id]);
    return Right<ApplicationError, NotebookRow>(made);
  }

  /// The statuses `statuses` answers.
  List<StatusChoice> statusChoices = <StatusChoice>[
    const StatusChoice(status: 'none', label: 'None'),
    const StatusChoice(status: 'active', label: 'Active'),
    const StatusChoice(status: 'onHold', label: 'On hold'),
    const StatusChoice(status: 'completed', label: 'Completed'),
    const StatusChoice(status: 'dropped', label: 'Dropped'),
  ];

  /// The notebook folds `visibleNotebooks` and `toggleFold` were asked for.
  final List<FoldState> foldsAsked = <FoldState>[];

  /// The notebook arrows tapped, with the notebook showing then.
  final List<(String, String?)> foldedTaps = <(String, String?)>[];

  /// The `tagPlaces` queries: all, selected, open levels, showing.
  final List<(bool, String?, List<String>, String?)> tagQueries =
      <(bool, String?, List<String>, String?)>[];

  /// The actions asked for, by name and argument.
  final List<(String, String?)> adminActions = <(String, String?)>[];

  /// How many notes `emptyTrash` answers.
  int emptyTrashCount = 0;

  /// What `noteLink` answers.
  CopiedLink link = const CopiedLink(
    text: '[[Design sync]]',
    message: 'Link copied.',
  );

  @override
  Future<Either<ApplicationError, List<StatusChoice>>> statuses() async =>
      Right<ApplicationError, List<StatusChoice>>(statusChoices);

  @override
  Future<Either<ApplicationError, int>> emptyTrash() async {
    adminActions.add(('emptyTrash', null));
    return Right<ApplicationError, int>(emptyTrashCount);
  }

  @override
  Future<Either<ApplicationError, Saved>> duplicate(String id) async {
    adminActions.add(('duplicate', id));
    return const Right<ApplicationError, Saved>(
      Saved(id: 'n:made', rev: 'r2'),
    );
  }

  @override
  Future<Either<ApplicationError, CopiedLink>> noteLink(
    String id, {
    required bool kanso,
  }) async {
    adminActions.add(('noteLink', id));
    return Right<ApplicationError, CopiedLink>(link);
  }

  @override
  Future<Either<ApplicationError, List<VisibleNotebook>>> visibleNotebooks(
    FoldState folds, {
    String? showing,
  }) async {
    foldsAsked.add(folds);
    final rows = notebookRows
        .where((row) => row.role != NotebookRole.templates)
        .toList();
    final showed = _visibleRows(rows, folds);
    return Right<ApplicationError, List<VisibleNotebook>>(showed);
  }

  /// The rows the fixture shows with [folds], from the notebooks' depths; a
  /// cut-down copy of the engine's rule, enough for widget tests.
  List<VisibleNotebook> _visibleRows(List<NotebookRow> rows, FoldState folds) {
    final open = folds.open.toSet();
    final shown = <VisibleNotebook>[];
    var inside = -1;
    for (var at = 0; at < rows.length; at++) {
      final row = rows[at];
      if (inside >= 0 && row.depth > inside) {
        continue;
      }
      inside = -1;
      final isOpen = open.contains(row.id);
      shown.add(
        VisibleNotebook(row: row, open: isOpen, parent: row.hasChildren),
      );
      if (row.hasChildren && !isOpen) {
        inside = row.depth;
      }
    }
    return shown;
  }

  @override
  Future<Either<ApplicationError, FoldState>> toggleFold(
    FoldState folds,
    String id, {
    String? showing,
  }) async {
    foldedTaps.add((id, showing));
    final open = <String>[...folds.open];
    if (!open.remove(id)) {
      open.add(id);
    }
    return Right<ApplicationError, FoldState>(
      FoldState(
        open: open,
        foldedAt: folds.foldedAt,
        folded: folds.folded,
      ),
    );
  }

  @override
  Future<Either<ApplicationError, TagPlaces>> tagPlaces({
    required bool all,
    required List<String> openPaths,
    String? selected,
    String? showing,
  }) async {
    tagQueries.add((all, selected, openPaths, showing));
    final chips = <TagRow>[
      for (final tag in tagRows)
        if (!tag.name.contains('/')) tag,
    ];
    final tree = <TagTreeRow>[];
    for (final tag in tagRows) {
      final parts = tag.name.split('/');
      if (parts.length < 2) {
        continue;
      }
      for (var level = 1; level < parts.length; level++) {
        final path = parts.take(level).join('/');
        if (tree.any((row) => row.path == path)) {
          continue;
        }
        tree.add(
          TagTreeRow(
            path: path,
            label: parts[level - 1],
            count: tag.count,
            depth: level - 1,
            parent: true,
            open: openPaths.contains(path.toLowerCase()),
          ),
        );
      }
      tree.add(
        TagTreeRow(
          path: tag.name,
          label: parts.last,
          tagId: tag.id,
          count: tag.count,
          depth: parts.length - 1,
          parent: false,
          open: false,
        ),
      );
    }
    return Right<ApplicationError, TagPlaces>(
      TagPlaces(chips: chips, more: false, tree: tree),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> renameNotebook(
    String id,
    String name,
  ) async {
    adminActions.add(('renameNotebook', '$id|$name'));
    notebookRows = <NotebookRow>[
      for (final row in notebookRows)
        if (row.id == id)
          NotebookRow(
            id: row.id,
            name: name,
            parentId: row.parentId,
            count: row.count,
            locked: row.locked,
            icon: row.icon,
            color: row.color,
            role: row.role,
            depth: row.depth,
            hasChildren: row.hasChildren,
          )
        else
          row,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> deleteNotebook(String id) async {
    adminActions.add(('deleteNotebook', id));
    notebookRows = <NotebookRow>[
      for (final row in notebookRows)
        if (row.id != id) row,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setNotebookIcon(
    String id,
    String? icon,
  ) async {
    adminActions.add(('setNotebookIcon', '$id|$icon'));
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setNotebookColor(
    String id,
    String? color,
  ) async {
    adminActions.add(('setNotebookColor', '$id|$color'));
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, TagRow>> createTag(String name) async {
    adminActions.add(('createTag', name));
    final made = TagRow(
      id: 'tag:made-${tagRows.length}',
      name: name,
      color: 0,
      count: 0,
    );
    tagRows = <TagRow>[...tagRows, made];
    return Right<ApplicationError, TagRow>(made);
  }

  @override
  Future<Either<ApplicationError, Unit>> renameTag(
    String id,
    String name,
  ) async {
    adminActions.add(('renameTag', '$id|$name'));
    tagRows = <TagRow>[
      for (final tag in tagRows)
        if (tag.id == id) TagRow(id: tag.id, name: name, color: tag.color,
            count: tag.count) else tag,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> deleteTag(String id) async {
    adminActions.add(('deleteTag', id));
    tagRows = <TagRow>[
      for (final tag in tagRows)
        if (tag.id != id) tag,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setTagColor(
    String id,
    String color,
  ) async {
    adminActions.add(('setTagColor', '$id|$color'));
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, List<IconChoice>>> notebookIcons(
    String query,
  ) async => const Right<ApplicationError, List<IconChoice>>(<IconChoice>[
    IconChoice(name: 'notebook', label: 'Notebook'),
    IconChoice(name: 'palette', label: 'Palette'),
  ]);

  @override
  Future<Either<ApplicationError, List<ColorChoice>>> notebookColors() async =>
      const Right<ApplicationError, List<ColorChoice>>(<ColorChoice>[
        ColorChoice(name: 'Default', rgb: 0),
        ColorChoice(name: 'Red', rgb: 0xEF4444),
      ]);

  @override
  Future<Either<ApplicationError, List<ColorChoice>>> tagColors() async =>
      const Right<ApplicationError, List<ColorChoice>>(<ColorChoice>[
        ColorChoice(name: 'default', rgb: 0),
        ColorChoice(name: 'red', rgb: 0xEF4444),
      ]);

  @override
  String? parseColor(String text) {
    final trimmed = text.trim();
    final digits = trimmed.startsWith('#') ? trimmed.substring(1) : trimmed;
    if (digits.isEmpty || !RegExp(r'^[0-9a-fA-F]+$').hasMatch(digits)) {
      return null;
    }
    if (digits.length == 6) {
      return '#${digits.toLowerCase()}';
    }
    if (digits.length == 3) {
      final doubled = digits.split('').map((c) => '$c$c').join();
      return '#${doubled.toLowerCase()}';
    }
    return null;
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
