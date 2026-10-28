// Kanso — journal_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/journal/state/journal/journal_bloc.dart';

import '../../helpers/fixture_journal_repository.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late FixtureJournalRepository fixture;

  setUp(() => fixture = FixtureJournalRepository());

  JournalBloc build() => JournalBloc(fixture, fixture);

  /// The bloc handles events on the subscription zone its constructor made,
  /// so the clock has to be pinned where it is built, not in `act`.
  JournalBloc buildAtNoon() =>
      withClock(Clock.fixed(DateTime(2026, 10, 26, 12)), build);

  group('JournalBloc', () {
    blocTest<JournalBloc, JournalState>(
      'today opens the note the library makes for today',
      build: buildAtNoon,
      act: (bloc) async {
        bloc.add(const JournalEvent.todayRequested());
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      expect: () => <Matcher>[
        isA<JournalState>().having((s) => s.working, 'working', true),
        isA<JournalState>().having(
          (s) => s.opened?.id,
          'opened',
          'note:daily-2026-10-26',
        ),
      ],
      verify: (_) => expect(fixture.calls, <String>['today']),
    );

    blocTest<JournalBloc, JournalState>(
      'stepping back counts from the open daily note',
      build: build,
      act: (bloc) async {
        bloc.add(
          const JournalEvent.stepRequested(
            openNote: 'note:daily-2026-10-01',
            forward: false,
          ),
        );
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      verify: (bloc) => expect(bloc.state.opened?.id, 'note:daily-2026-09-30'),
    );

    blocTest<JournalBloc, JournalState>(
      'stepping with no note open counts from today',
      build: buildAtNoon,
      act: (bloc) async {
        bloc.add(
          const JournalEvent.stepRequested(openNote: null, forward: true),
        );
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      verify: (bloc) => expect(bloc.state.opened?.id, 'note:daily-2026-10-27'),
    );

    blocTest<JournalBloc, JournalState>(
      'a day of the calendar opens that day',
      build: build,
      act: (bloc) async {
        bloc.add(
          const JournalEvent.dayRequested(
            DayDate(year: 2026, month: 10, day: 12),
          ),
        );
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      verify: (bloc) {
        expect(bloc.state.opened?.id, 'note:daily-2026-10-12');
        expect(fixture.calls, <String>['day:2026-10-12']);
      },
    );

    blocTest<JournalBloc, JournalState>(
      'a template makes a note in the notebook asked for',
      build: build,
      act: (bloc) async {
        bloc.add(
          const JournalEvent.fromTemplateRequested(
            templateId: 'note:tpl-daily',
            notebookId: 'book:work',
          ),
        );
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      verify: (bloc) {
        expect(bloc.state.opened?.id, 'note:from-note:tpl-daily');
        expect(fixture.calls, <String>['from:note:tpl-daily@book:work']);
      },
    );

    blocTest<JournalBloc, JournalState>(
      'a new template opens a new template',
      build: build,
      act: (bloc) async {
        bloc.add(const JournalEvent.newTemplateRequested());
        await bloc.stream.firstWhere((state) => state.opened != null);
      },
      verify: (bloc) => expect(bloc.state.opened?.id, 'note:new-template'),
    );

    blocTest<JournalBloc, JournalState>(
      'a failure is kept until it is dismissed, and opens nothing',
      build: () {
        fixture.failure = _failure;
        return build();
      },
      act: (bloc) async {
        bloc.add(const JournalEvent.todayRequested());
        await bloc.stream.firstWhere((state) => state.error != null);
        bloc.add(const JournalEvent.errorDismissed());
      },
      expect: () => <Matcher>[
        isA<JournalState>().having((s) => s.working, 'working', true),
        isA<JournalState>()
            .having((s) => s.error, 'error', _failure)
            .having((s) => s.opened, 'opened', isNull),
        isA<JournalState>().having((s) => s.error, 'error', isNull),
      ],
    );
  });
}
