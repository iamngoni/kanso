// Kanso — notebook_home_bloc_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/notebook_home/state/notebook_home/notebook_home_bloc.dart';

import '../../helpers/fake_notebook_home_repository.dart';

// no-prop-test: event ordering is about when calls happen, not about values.
void main() {
  late FakeNotebookHomeRepository repo;
  late NotebookHomeBloc bloc;

  setUp(() {
    repo = FakeNotebookHomeRepository(
      panelAnswer: homePanel(<PageSection>[
        homeSection('about', 'About', ai: true),
      ]),
    );
    bloc = NotebookHomeBloc(repo);
  });

  tearDown(() => bloc.close());

  group('NotebookHomeBloc', () {
    test('the first open waits for the words', () async {
      repo.wordsGate = Completer<void>();
      bloc.add(const NotebookHomeEvent.started('b1'));
      await pumpEventQueue();
      // The words are being read; the page has not been asked for.
      expect(repo.calls, <String>['words']);
      expect(bloc.state.panel, isNull);

      repo.wordsGate!.complete();
      await pumpEventQueue();
      expect(repo.calls.take(2), <String>['words', 'open']);
      expect(bloc.state.words, isNotNull);
      expect(bloc.state.status, LoadStatus.ready);
    });

    test('events that touch the page never interleave', () async {
      repo.openGate = Completer<void>();
      bloc
        ..add(const NotebookHomeEvent.started('b1'))
        ..add(const NotebookHomeEvent.pinned('about', on: true))
        ..add(const NotebookHomeEvent.hidden('about', hidden: true))
        ..add(const NotebookHomeEvent.allShown('open_tasks'));
      await pumpEventQueue();
      // The open is still running: nothing behind it has started.
      expect(repo.calls, <String>['words', 'open']);

      repo.openGate!.complete();
      await pumpEventQueue();
      final order = repo.calls.where((c) => c != 'map').toList();
      expect(order, <String>[
        'words',
        'open',
        'pin:about:true',
        'hide:about:true',
        'showAll:open_tasks',
      ]);
    });

    test('a refresh holds the events behind it until it is done', () async {
      repo.refreshGate = Completer<void>();
      bloc.add(const NotebookHomeEvent.started('b1'));
      await pumpEventQueue();
      bloc
        ..add(const NotebookHomeEvent.sectionRefreshRequested('about'))
        ..add(const NotebookHomeEvent.pinned('about', on: true));
      await pumpEventQueue();
      expect(bloc.state.working, <String>{'about'});
      expect(repo.calls.where((c) => c.startsWith('pin')), isEmpty);

      repo.refreshGate!.complete();
      await pumpEventQueue();
      expect(bloc.state.working, isEmpty);
      expect(repo.calls.where((c) => c.startsWith('pin')), hasLength(1));
    });

    test(
      'sections that are due are refreshed once, on the first open',
      () async {
        repo.panelAnswer = homePanel(
          <PageSection>[
            homeSection('about', 'About', ai: true),
            homeSection('decisions', 'Decisions', ai: true),
          ],
          aiOn: true,
          due: <String>['about', 'decisions'],
        );
        bloc.add(const NotebookHomeEvent.started('b1'));
        await pumpEventQueue();
        expect(repo.calls.where((c) => c.startsWith('refresh:')), <String>[
          'refresh:about',
          'refresh:decisions',
        ]);

        // Coming back to the page does not ask again.
        bloc.add(const NotebookHomeEvent.reopened());
        await pumpEventQueue();
        expect(
          repo.calls.where((c) => c.startsWith('refresh:')),
          hasLength(2),
        );
      },
    );

    test('the map comes after the sections and holds nothing up', () async {
      repo.mapAnswer = const HomeMap(
        width: 320,
        height: 200,
        nodes: <HomeMapNode>[
          HomeMapNode(
            id: 'n1',
            title: 'One',
            x: 10,
            y: 10,
            radius: 4,
            degree: 1,
            stubs: 0,
          ),
        ],
        edges: <HomeMapEdge>[],
      );
      bloc.add(const NotebookHomeEvent.started('b1'));
      await pumpEventQueue();
      expect(
        repo.calls.indexOf('map'),
        greaterThan(repo.calls.indexOf('open')),
      );
      expect(bloc.state.panel, isNotNull);
      expect(bloc.state.map?.nodes.single.id, 'n1');
    });

    test('ticking a task changes it and reads the page again', () async {
      bloc.add(const NotebookHomeEvent.started('b1'));
      await pumpEventQueue();
      const task = PageTask(noteId: 'n1', line: 3, text: 'Pay', checked: false);
      bloc.add(const NotebookHomeEvent.taskTicked(task, checked: true));
      await pumpEventQueue();
      expect(repo.ticked, task);
      expect(repo.tickedTo, isTrue);
      expect(repo.calls.where((c) => c == 'open'), hasLength(2));
    });

    test('a gone notebook is a failed page', () async {
      repo.panelAnswer = null;
      bloc.add(const NotebookHomeEvent.started('b1'));
      await pumpEventQueue();
      expect(bloc.state.status, LoadStatus.failed);
    });
  });
}
