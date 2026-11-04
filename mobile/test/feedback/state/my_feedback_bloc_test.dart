// Kanso — my_feedback_bloc_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/feedback/models/data/feedback_entry.dart';
import 'package:kanso/feedback/state/my_feedback/my_feedback_bloc.dart';

import '../../helpers/fixture_feedback_repository.dart';

void main() {
  late FixtureFeedbackRepository repository;

  setUp(() {
    repository = FixtureFeedbackRepository(
      entries: <FeedbackEntry>[
        sampleFeedbackEntry(
          id: 'fb_b',
          chip: FeedbackChip.releasedIn,
          release: '0.9.2',
          fresh: true,
        ),
        sampleFeedbackEntry(
          chip: FeedbackChip.filedAs,
          number: 241,
        ),
      ],
    );
  });

  MyFeedbackBloc build() => MyFeedbackBloc(repository);

  group('MyFeedbackBloc', () {
    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'Settings reads the phone and asks the server nothing',
      build: build,
      act: (bloc) => bloc.add(const MyFeedbackEvent.started()),
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.entries, hasLength(2));
        expect(repository.refreshes, <bool>[false]);
        expect(bloc.state.freshRelease, '0.9.2');
        expect(bloc.state.newest?.id, 'fb_b');
      },
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'a launch asks the server only when the 6 hours are up',
      build: build,
      act: (bloc) {
        bloc.add(const MyFeedbackEvent.started(launch: true));
      },
      verify: (_) => expect(repository.refreshes, <bool>[false]),
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'a launch that is due asks the server',
      build: () {
        repository.due = true;
        return build();
      },
      act: (bloc) => bloc.add(const MyFeedbackEvent.started(launch: true)),
      verify: (_) => expect(repository.refreshes, <bool>[true]),
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'opening My feedback asks the server once',
      build: build,
      act: (bloc) => bloc.add(const MyFeedbackEvent.refreshRequested()),
      verify: (bloc) {
        expect(repository.refreshes, <bool>[true]);
        expect(bloc.state.refreshing, isFalse);
      },
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'Delete removes the report, then reads the list again',
      build: build,
      act: (bloc) async {
        bloc
          ..add(const MyFeedbackEvent.started())
          ..add(const MyFeedbackEvent.deleteRequested('fb_a'));
      },
      verify: (bloc) {
        expect(repository.deleted, <String>['fb_a']);
        expect(bloc.state.entries.map((e) => e.id), <String>['fb_b']);
      },
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'coming back to the front sends what is waiting',
      build: build,
      act: (bloc) => bloc.add(const MyFeedbackEvent.resumed()),
      verify: (_) => expect(repository.flushes, 1),
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      'looking at the released reports marks them seen',
      build: build,
      act: (bloc) => bloc.add(const MyFeedbackEvent.seen()),
      verify: (_) => expect(repository.seen, 1),
    );

    blocTest<MyFeedbackBloc, MyFeedbackState>(
      "a library that can't be read fails the page",
      build: () {
        final broken = _Broken();
        return MyFeedbackBloc(broken);
      },
      act: (bloc) => bloc.add(const MyFeedbackEvent.started()),
      verify: (bloc) => expect(bloc.state.status, LoadStatus.failed),
    );
  });
}

class _Broken extends FixtureFeedbackRepository {
  @override
  Future<List<FeedbackEntry>?> entries({bool refresh = false}) async => null;
}
