// Kanso — fake_editor_document.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/cancelable_stream.dart';
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';

/// An [EditorDocument] with no core behind it: a text in memory, the calls
/// it was made, and answers a test sets, for what the real core does not do
/// on demand (an older text, a lock closing, a late batch of colours).
class FakeEditorDocument implements EditorDocument {
  FakeEditorDocument({this.text = '', this.title = ''});

  String text;
  String title;
  int version = 1;
  HistoryState history = const HistoryState(canUndo: false, canRedo: false);
  SaveState save = const SaveState.saved(rev: 'r1', atMs: 0);
  GoalSetting wordGoal = const GoalSetting.default_();
  int? defaultGoal;
  List<Property> properties = const <Property>[];
  Completion? offered;
  LinkTarget? link;
  EditorContext context = const EditorContext(
    inList: false,
    inTable: false,
    inCode: false,
    completionHere: false,
    history: HistoryState(canUndo: false, canRedo: false),
  );

  /// What the next call to [apply] fails with, once.
  ApplicationError? failNextApply;

  /// What every call fails with while it is set.
  ApplicationError? failAll;

  /// What the named calls (`flush`, `keepMine`, `useTheirs`, `close`,
  /// `completion`, `setProperty`, `removeProperty`, `snapshot`) fail with.
  final Map<String, ApplicationError> failures = <String, ApplicationError>{};

  /// What attaching the file with a given name fails with.
  final Map<String, ApplicationError> attachFailures =
      <String, ApplicationError>{};

  /// What the next completion is, once, for a test that changes it between
  /// calls.
  Future<Completion?> Function()? completionBuilder;

  /// Whether `events()` was listened to, and still is.
  bool get listening => events_.hasListener;

  /// The version the next batch of colours says it is for.
  int? batchVersion;

  /// What the next batch of colours holds.
  List<EditorRun> batchRuns = const <EditorRun>[];

  /// The outcome of the next command, undo or redo; null for none.
  EditOutcome? nextOutcome;

  final List<TextChange> applied = <TextChange>[];
  final List<EditCommand> commands = <EditCommand>[];
  final List<(Range16, bool)> selections = <(Range16, bool)>[];
  final List<String> calls = <String>[];
  int runRequests = 0;
  final StreamController<EditorEvent> events_ =
      StreamController<EditorEvent>.broadcast();

  EditorSnapshot get current => EditorSnapshot(
    id: 'note:fake',
    title: title,
    text: text,
    rev: 'r1',
    lock: const LockState.plain(),
    properties: properties,
    version: version,
    selection: Range16(start: text.length, end: text.length),
    history: history,
    save: save,
    wordGoal: wordGoal,
  );

  @override
  Future<Either<ApplicationError, int?>> effectiveWordGoal() async {
    final failure = failed<int?>('effectiveWordGoal');
    if (failure != null) {
      return failure;
    }
    return Right<ApplicationError, int?>(switch (wordGoal) {
      GoalSetting_Default() => defaultGoal,
      GoalSetting_Off() => null,
      GoalSetting_Words(:final field0) => field0,
    });
  }

  @override
  Future<Either<ApplicationError, int?>> defaultWordGoal() async =>
      failed<int?>('defaultWordGoal') ??
      Right<ApplicationError, int?>(defaultGoal);

  @override
  Future<Either<ApplicationError, Unit>> setWordGoal(
    GoalSetting setting,
  ) async {
    calls.add('setWordGoal $setting');
    final failure = failed<Unit>('setWordGoal');
    if (failure != null) {
      return failure;
    }
    wordGoal = setting;
    return const Right<ApplicationError, Unit>(unit);
  }

  Either<ApplicationError, T> failure<T>() =>
      Left<ApplicationError, T>(failAll!);

  /// The failure of the call [name], if one is set.
  Either<ApplicationError, T>? failed<T>(String name) {
    final error = failures[name] ?? failAll;
    return error == null ? null : Left<ApplicationError, T>(error);
  }

  @override
  Future<Either<ApplicationError, EditorSnapshot>> snapshot() async {
    calls.add('snapshot');
    return failed<EditorSnapshot>('snapshot') ?? Right(current);
  }

  @override
  Either<ApplicationError, EditOutcome> apply(TextChange change) {
    applied.add(change);
    if (failAll != null) {
      return failure();
    }
    final error = failNextApply;
    if (error != null) {
      failNextApply = null;
      return Left(error);
    }
    text = text.replaceRange(change.start, change.end, change.text);
    version++;
    history = const HistoryState(canUndo: true, canRedo: false);
    return Right(
      EditOutcome(
        version: version,
        selection: change.selection,
        history: history,
      ),
    );
  }

  @override
  Either<ApplicationError, EditorContext> select(
    Range16 selection, {
    required bool composing,
  }) {
    selections.add((selection, composing));
    return failAll != null
        ? failure()
        : Right(
            EditorContext(
              inList: context.inList,
              inTable: context.inTable,
              inCode: context.inCode,
              completionHere: context.completionHere,
              history: history,
            ),
          );
  }

  @override
  Either<ApplicationError, EditOutcome?> command(EditCommand command) {
    commands.add(command);
    return failAll != null ? failure() : Right(_take());
  }

  @override
  Either<ApplicationError, EditOutcome?> undo() {
    calls.add('undo');
    return failAll != null ? failure() : Right(_take());
  }

  @override
  Either<ApplicationError, EditOutcome?> redo() {
    calls.add('redo');
    return failAll != null ? failure() : Right(_take());
  }

  EditOutcome? _take() {
    final outcome = nextOutcome;
    nextOutcome = null;
    if (outcome != null) {
      version = outcome.version;
      history = outcome.history;
      final replace = outcome.replace;
      if (replace != null) {
        text = text.replaceRange(replace.start, replace.end, replace.text);
      }
    }
    return outcome;
  }

  @override
  Either<ApplicationError, Unit> setTitle(String title) {
    this.title = title;
    calls.add('setTitle');
    return const Right(unit);
  }

  @override
  Future<Either<ApplicationError, RunBatch>> styledRuns({
    required int start,
    required int end,
    required String themeId,
  }) async {
    runRequests++;
    if (failAll != null) {
      return failure();
    }
    final asked = batchVersion ?? version;
    final runs = batchRuns;
    if (batchVersion != null) {
      // A batch for another version is sent once.
      batchRuns = const <EditorRun>[];
    }
    batchVersion = null;
    return Right(RunBatch(version: asked, runs: runs));
  }

  @override
  Future<Either<ApplicationError, Completion?>> completion() async {
    calls.add('completion');
    final build = completionBuilder;
    if (build != null) {
      return Right(await build());
    }
    return failed<Completion?>('completion') ?? Right(offered);
  }

  @override
  Future<Either<ApplicationError, LinkTarget?>> linkAt(int offset) async {
    calls.add('linkAt $offset');
    return failAll != null ? failure() : Right(link);
  }

  @override
  Future<Either<ApplicationError, List<Property>>> setProperty({
    required String key,
    required String value,
    int? index,
  }) async {
    calls.add('setProperty $index $key $value');
    return failed<List<Property>>('setProperty') ?? Right(properties);
  }

  @override
  Future<Either<ApplicationError, List<Property>>> removeProperty(
    int index,
  ) async {
    calls.add('removeProperty $index');
    return failed<List<Property>>('removeProperty') ?? Right(properties);
  }

  @override
  Future<Either<ApplicationError, SaveState>> flush() async {
    calls.add('flush');
    return failed<SaveState>('flush') ?? Right(save);
  }

  @override
  Future<Either<ApplicationError, SaveState>> keepMine() async {
    calls.add('keepMine');
    return failed<SaveState>('keepMine') ?? Right(save);
  }

  @override
  Future<Either<ApplicationError, EditorSnapshot>> useTheirs() async {
    calls.add('useTheirs');
    return failed<EditorSnapshot>('useTheirs') ?? Right(current);
  }

  @override
  Future<Either<ApplicationError, SaveState>> close() async {
    calls.add('close');
    return failed<SaveState>('close') ?? Right(save);
  }

  @override
  Stream<EditorEvent> events() => cancelableStream(events_.stream);

  /// The names of the files attached, in turn.
  final List<String> attachedFiles = <String>[];

  /// Attaching appends `![](inkdrop://file:<name>)` to the text, as a
  /// paragraph of its own, or fails with the call's failure.
  @override
  Future<Either<ApplicationError, EditOutcome>> attach(ShareFile file) async {
    calls.add('attach ${file.name}');
    final refused = attachFailures[file.name];
    if (refused != null) {
      return Left<ApplicationError, EditOutcome>(refused);
    }
    final failure = failed<EditOutcome>('attach');
    if (failure != null) {
      return failure;
    }
    attachedFiles.add(file.name);
    final start = text.length;
    final block =
        '${text.isEmpty ? '' : '\n\n'}![](inkdrop://file:${file.name})';
    text = '$text$block';
    version++;
    history = const HistoryState(canUndo: true, canRedo: false);
    return Right(
      EditOutcome(
        version: version,
        replace: Replacement(start: start, end: start, text: block),
        selection: Range16(start: text.length, end: text.length),
        history: history,
      ),
    );
  }

  /// The words the note's comments are on.
  List<Range16> marks = const <Range16>[];

  /// Where each comment's words are, by ID.
  Map<String, Range16> commentWords = <String, Range16>{};

  /// When set, commenting fails with it.
  ApplicationError? commentFailure;

  @override
  Future<Either<ApplicationError, String>> commentOnSelection(
    String body,
  ) async {
    calls.add('commentOnSelection $body');
    final failure = commentFailure;
    return failure == null
        ? const Right<ApplicationError, String>('c_new')
        : Left<ApplicationError, String>(failure);
  }

  @override
  Future<Either<ApplicationError, List<Range16>>> commentMarks() async {
    calls.add('commentMarks');
    return Right<ApplicationError, List<Range16>>(marks);
  }

  @override
  Future<Either<ApplicationError, Range16?>> selectComment(
    String commentId,
  ) async {
    calls.add('selectComment $commentId');
    return Right<ApplicationError, Range16?>(commentWords[commentId]);
  }

  /// Other people's carets and selections.
  List<CursorMark> cursors = const <CursorMark>[];

  @override
  Future<Either<ApplicationError, List<CursorMark>>> remoteCursors() async {
    calls.add('remoteCursors');
    return Right<ApplicationError, List<CursorMark>>(cursors);
  }
}
