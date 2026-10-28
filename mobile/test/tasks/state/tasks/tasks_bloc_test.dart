// Kanso — tasks_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/tasks/models/data/tasks_notice.dart';
import 'package:kanso/tasks/models/data/tasks_status.dart';
import 'package:kanso/tasks/state/tasks/tasks_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

TaskItem _first = mockTaskBoard.groups.first.items.first;

void main() {
  late MockLibraryRepository library;
  late MockHapticsRepository haptics;
  late MockUndoNoticeBloc undoNotice;
  late StreamController<List<String>> changes;

  setUpAll(() {
    registerFallbackValue(const TaskListQuery(showDone: false, kept: []));
    registerFallbackValue(const UndoNoticeEvent.dismissed());
  });

  setUp(() {
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    haptics = MockHapticsRepository();
    undoNotice = MockUndoNoticeBloc();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.tasks(any())).thenAnswer(
      (_) async => const Right<ApplicationError, TaskBoard>(mockTaskBoard),
    );
    when(
      () => library.setTask(
        noteId: any(named: 'noteId'),
        line: any(named: 'line'),
        text: any(named: 'text'),
        checked: any(named: 'checked'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, Ticked>(
        Ticked.changed(rev: 'r2', undo: 7, message: 'Ticked off “Send Sam”'),
      ),
    );
    when(() => haptics.selection()).thenAnswer((_) async {});
  });

  TasksBloc build() => TasksBloc(library, haptics, undoNotice);

  TypeMatcher<TasksState> ready() => isA<TasksState>().having(
    (state) => state.status,
    'status',
    TasksStatus.ready,
  );

  group('TasksBloc', () {
    test('starts loading, with nothing', () {
      final state = build().state;
      expect(state.status, TasksStatus.loading);
      expect(state.count, 0);
    });

    blocTest<TasksBloc, TasksState>(
      'started reads the tasks of every notebook, without done ones',
      build: build,
      act: (bloc) => bloc.add(const TasksEvent.started()),
      expect: () => <Matcher>[
        isA<TasksState>().having(
          (state) => state.status,
          'status',
          TasksStatus.loading,
        ),
        ready()
            .having((state) => state.board, 'board', mockTaskBoard)
            .having((state) => state.count, 'count', 6),
      ],
      verify: (_) {
        final query =
            verify(() => library.tasks(captureAny())).captured.single
                as TaskListQuery;
        expect(query.notebookId, isNull);
        expect(query.showDone, isFalse);
        expect(query.kept, isEmpty);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'choosing a notebook reads only its tasks',
      build: build,
      act: (bloc) async {
        bloc.add(const TasksEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == TasksStatus.ready,
        );
        bloc.add(const TasksEvent.notebookChosen('b-work'));
      },
      verify: (bloc) {
        final queries = verify(
          () => library.tasks(captureAny()),
        ).captured.cast<TaskListQuery>();
        expect(queries.last.notebookId, 'b-work');
        expect(bloc.state.notebookId, 'b-work');
      },
    );

    blocTest<TasksBloc, TasksState>(
      'showing completed reads the done tasks too',
      build: build,
      act: (bloc) async {
        bloc.add(const TasksEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == TasksStatus.ready,
        );
        bloc.add(const TasksEvent.showDoneChanged(showDone: true));
      },
      verify: (bloc) {
        final queries = verify(
          () => library.tasks(captureAny()),
        ).captured.cast<TaskListQuery>();
        expect(queries.last.showDone, isTrue);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'ticking writes once, keeps the task, shows the notice, reads again',
      build: build,
      act: (bloc) async {
        bloc.add(const TasksEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == TasksStatus.ready,
        );
        bloc.add(TasksEvent.ticked(_first));
        await bloc.stream.firstWhere((state) => state.kept.isNotEmpty);
        await Future<void>.delayed(Duration.zero);
      },
      verify: (bloc) {
        verify(
          () => library.setTask(
            noteId: _first.noteId,
            line: _first.line,
            // The text as the note has it, not as the list shows it.
            text: _first.sourceText,
            checked: true,
          ),
        ).called(1);
        expect(bloc.state.kept, <String>[_first.key]);
        final queries = verify(
          () => library.tasks(captureAny()),
        ).captured.cast<TaskListQuery>();
        expect(queries.last.kept, <String>[_first.key]);
        final event =
            verify(() => undoNotice.add(captureAny())).captured.single
                as UndoNoticeShownEvent;
        expect(event.ticket, 7);
        expect(event.message, 'Ticked off “Send Sam”');
        verify(() => haptics.selection()).called(1);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'clearing a ticked task writes it open and lets it go from the kept',
      build: build,
      seed: () => TasksState(
        status: TasksStatus.ready,
        board: mockTaskBoard,
        kept: <String>[_first.key],
      ),
      act: (bloc) => bloc.add(
        TasksEvent.ticked(
          TaskItem(
            key: _first.key,
            noteId: _first.noteId,
            noteTitle: _first.noteTitle,
            notebookId: _first.notebookId,
            notebookName: _first.notebookName,
            line: _first.line,
            text: _first.text,
            sourceText: _first.sourceText,
            checked: true,
          ),
        ),
      ),
      verify: (bloc) {
        verify(
          () => library.setTask(
            noteId: _first.noteId,
            line: _first.line,
            text: _first.sourceText,
            checked: false,
          ),
        ).called(1);
        expect(bloc.state.kept, isEmpty);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'a task that moved says so and the list is read again',
      build: build,
      setUp: () =>
          when(
            () => library.setTask(
              noteId: any(named: 'noteId'),
              line: any(named: 'line'),
              text: any(named: 'text'),
              checked: any(named: 'checked'),
            ),
          ).thenAnswer(
            (_) async =>
                const Right<ApplicationError, Ticked>(Ticked.missing()),
          ),
      act: (bloc) => bloc.add(TasksEvent.ticked(_first)),
      expect: () => <Matcher>[
        isA<TasksState>().having(
          (state) => state.notice,
          'notice',
          TasksNotice.moved,
        ),
        ready().having((state) => state.notice, 'notice', TasksNotice.moved),
      ],
      verify: (_) {
        verifyNever(() => undoNotice.add(any()));
        verify(() => library.tasks(any())).called(1);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'a task that was already so changes nothing and shows no notice',
      build: build,
      setUp: () =>
          when(
            () => library.setTask(
              noteId: any(named: 'noteId'),
              line: any(named: 'line'),
              text: any(named: 'text'),
              checked: any(named: 'checked'),
            ),
          ).thenAnswer(
            (_) async =>
                const Right<ApplicationError, Ticked>(Ticked.unchanged()),
          ),
      act: (bloc) => bloc.add(TasksEvent.ticked(_first)),
      verify: (bloc) {
        verifyNever(() => undoNotice.add(any()));
        expect(bloc.state.kept, isEmpty);
        expect(bloc.state.notice, isNull);
      },
    );

    blocTest<TasksBloc, TasksState>(
      'a failed tick says so, and a locked note says that',
      build: build,
      setUp: () =>
          when(
            () => library.setTask(
              noteId: any(named: 'noteId'),
              line: any(named: 'line'),
              text: any(named: 'text'),
              checked: any(named: 'checked'),
            ),
          ).thenAnswer(
            (_) async => const Left<ApplicationError, Ticked>(
              ApplicationError.locked(message: 'Locked.'),
            ),
          ),
      act: (bloc) => bloc.add(TasksEvent.ticked(_first)),
      expect: () => <Matcher>[
        isA<TasksState>().having(
          (state) => state.notice,
          'notice',
          TasksNotice.locked,
        ),
      ],
      verify: (_) => verifyNever(() => undoNotice.add(any())),
    );

    blocTest<TasksBloc, TasksState>(
      'dismissing the notice clears it',
      build: build,
      seed: () => const TasksState(notice: TasksNotice.failed),
      act: (bloc) => bloc.add(const TasksEvent.noticeDismissed()),
      expect: () => <Matcher>[
        isA<TasksState>().having((state) => state.notice, 'notice', isNull),
      ],
    );

    blocTest<TasksBloc, TasksState>(
      'a failed reading fails, and trying again reads',
      build: build,
      setUp: () => when(() => library.tasks(any())).thenAnswer(
        (_) async => const Left<ApplicationError, TaskBoard>(_failure),
      ),
      act: (bloc) async {
        bloc.add(const TasksEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == TasksStatus.failed,
        );
        when(() => library.tasks(any())).thenAnswer(
          (_) async => const Right<ApplicationError, TaskBoard>(mockTaskBoard),
        );
        bloc.add(const TasksEvent.retried());
      },
      verify: (bloc) {
        expect(bloc.state.status, TasksStatus.ready);
        expect(bloc.state.error, isNull);
        expect(bloc.state.count, 6);
      },
    );

    test('reads again when the library changes, once per burst', () async {
      final bloc = build()..add(const TasksEvent.started());
      await bloc.stream.firstWhere(
        (state) => state.status == TasksStatus.ready,
      );
      clearInteractions(library);
      changes
        ..add(<String>['n1'])
        ..add(<String>['n2'])
        ..add(<String>['n3']);
      await Future<void>.delayed(tasksDebounce * 3);
      verify(() => library.tasks(any())).called(1);
      await bloc.close();
    });

    test('stops listening to the library when closed', () async {
      final bloc = build()..add(const TasksEvent.started());
      await bloc.stream.firstWhere(
        (state) => state.status == TasksStatus.ready,
      );
      expect(changes.hasListener, isTrue);
      await bloc.close();
      expect(changes.hasListener, isFalse);
    });
  });
}
