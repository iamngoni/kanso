// Kanso — resurface_bloc_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as g;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/discovery/models/repos/abs/discovery_repository.dart';
import 'package:kanso/note/state/resurface/resurface_bloc.dart';
import 'package:mocktail/mocktail.dart';

class _MockDiscovery extends Mock implements DiscoveryRepository {}

class _MockPreferences extends Mock implements PreferencesRepository {}

ResurfaceRow _row(String id) => ResurfaceRow(
  id: id,
  title: 'Note $id',
  book: 'Home',
  when: 'March 2024',
  snippet: '…the pressure valve…',
);

ResurfaceState _state({
  bool enabled = true,
  List<ResurfaceRow> rows = const <ResurfaceRow>[],
  List<OnThisDayRow> day = const <OnThisDayRow>[],
}) => ResurfaceState(
  noteId: 'note-a',
  enabled: enabled,
  rows: rows,
  day: day,
);

void main() {
  late _MockDiscovery discovery;
  late _MockPreferences preferences;

  ResurfaceBloc build() => ResurfaceBloc(discovery, preferences)
    ..settle = const Duration(milliseconds: 20)
    ..utcOffsetSecs = () => 0;

  setUpAll(() {
    registerFallbackValue(const Preferences());
  });

  setUp(() {
    discovery = _MockDiscovery();
    preferences = _MockPreferences();
    when(
      () => discovery.relatedEnabled(),
    ).thenAnswer((_) async => right(true));
    when(
      () => preferences.load(),
    ).thenAnswer((_) async => right(const Preferences()));
    when(
      () => discovery.onThisDay(
        any(),
        utcOffsetSecs: any(named: 'utcOffsetSecs'),
      ),
    ).thenAnswer((_) async => right(const <OnThisDayRow>[]));
    when(
      () => discovery.dismissRelated(any(), any()),
    ).thenAnswer((_) async => right(unit));
  });

  void answer(List<ResurfaceRow> rows) {
    when(
      () => discovery.resurface(
        id: any(named: 'id'),
        title: any(named: 'title'),
        body: any(named: 'body'),
        caret: any(named: 'caret'),
        closeness: any(named: 'closeness'),
        utcOffsetSecs: any(named: 'utcOffsetSecs'),
      ),
    ).thenAnswer((_) async => right(ResurfaceResult.found(rows: rows)));
  }

  blocTest<ResurfaceBloc, ResurfaceState>(
    'starts enabled when related notes and the setting are on, and lists '
    'the notes of this day on a daily note',
    setUp: () {
      when(
        () => discovery.onThisDay(
          'note-a',
          utcOffsetSecs: any(named: 'utcOffsetSecs'),
        ),
      ).thenAnswer(
        (_) async => right(const <OnThisDayRow>[
          OnThisDayRow(
            id: 'old',
            title: 'Walk',
            book: 'Journal',
            ago: '2 years ago',
          ),
        ]),
      );
    },
    build: build,
    act: (bloc) => bloc.add(const ResurfaceEvent.started('note-a')),
    expect: () => <ResurfaceState>[
      _state(),
      _state(
        day: const <OnThisDayRow>[
          OnThisDayRow(
            id: 'old',
            title: 'Walk',
            book: 'Journal',
            ago: '2 years ago',
          ),
        ],
      ),
    ],
  );

  blocTest<ResurfaceBloc, ResurfaceState>(
    'is off while related notes are, and asks nothing',
    setUp: () {
      when(
        () => discovery.relatedEnabled(),
      ).thenAnswer((_) async => right(false));
    },
    build: build,
    act: (bloc) async {
      bloc
        ..add(const ResurfaceEvent.started('note-a'))
        ..add(
          const ResurfaceEvent.edited(
            title: 'T',
            body: 'one two three four five six seven',
            caret: 3,
          ),
        );
      await Future<void>.delayed(const Duration(milliseconds: 80));
    },
    verify: (bloc) {
      expect(bloc.state.enabled, isFalse);
      verifyNever(
        () => discovery.resurface(
          id: any(named: 'id'),
          title: any(named: 'title'),
          body: any(named: 'body'),
          caret: any(named: 'caret'),
          closeness: any(named: 'closeness'),
          utcOffsetSecs: any(named: 'utcOffsetSecs'),
        ),
      );
    },
  );

  test(
    'asks only once the text has paused, and only for the last edit',
    () async {
      answer(<ResurfaceRow>[_row('a')]);
      final bloc = build()..add(const ResurfaceEvent.started('note-a'));
      await Future<void>.delayed(const Duration(milliseconds: 5));
      for (final text in <String>[
        'one',
        'one two',
        'one two three four five six',
      ]) {
        bloc.add(
          ResurfaceEvent.edited(title: 'T', body: text, caret: text.length),
        );
        await Future<void>.delayed(const Duration(milliseconds: 5));
      }
      // Typing: nothing has been asked yet.
      verifyNever(
        () => discovery.resurface(
          id: any(named: 'id'),
          title: any(named: 'title'),
          body: any(named: 'body'),
          caret: any(named: 'caret'),
          closeness: any(named: 'closeness'),
          utcOffsetSecs: any(named: 'utcOffsetSecs'),
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 80));
      verify(
        () => discovery.resurface(
          id: 'note-a',
          title: 'T',
          body: 'one two three four five six',
          caret: 27,
          closeness: 'default',
          utcOffsetSecs: 0,
        ),
      ).called(1);
      expect(bloc.state.rows.map((r) => r.id), <String>['a']);
      await bloc.close();
    },
  );

  test('an edit sent right after started is asked once it is loaded', () async {
    answer(<ResurfaceRow>[_row('a')]);
    final bloc = build()
      ..add(const ResurfaceEvent.started('note-a'))
      ..add(
        const ResurfaceEvent.edited(
          title: 'T',
          body: 'one two three four five six seven',
          caret: 3,
        ),
      );
    await Future<void>.delayed(const Duration(milliseconds: 120));
    expect(bloc.state.rows.map((r) => r.id), <String>['a']);
    await bloc.close();
  });

  test('the Settings changed since the note opened are picked up', () async {
    answer(<ResurfaceRow>[_row('a')]);
    var on = false;
    when(() => preferences.load()).thenAnswer(
      (_) async => right(Preferences(resurface: on)),
    );
    final bloc = build()..add(const ResurfaceEvent.started('note-a'));
    await Future<void>.delayed(const Duration(milliseconds: 10));
    expect(bloc.state.enabled, isFalse);
    on = true;
    bloc.add(
      const ResurfaceEvent.edited(
        title: 'T',
        body: 'one two three four five six seven',
        caret: 3,
      ),
    );
    await Future<void>.delayed(const Duration(milliseconds: 120));
    expect(bloc.state.enabled, isTrue);
    expect(bloc.state.rows.map((r) => r.id), <String>['a']);
    await bloc.close();
  });

  test('notes that stay stay in place, new ones come in at the end', () async {
    final bloc = build()..add(const ResurfaceEvent.started('note-a'));
    await Future<void>.delayed(const Duration(milliseconds: 5));
    Future<void> ask(List<ResurfaceRow> rows) async {
      answer(rows);
      bloc.add(
        const ResurfaceEvent.edited(
          title: 'T',
          body: 'one two three four five six',
          caret: 3,
        ),
      );
      await Future<void>.delayed(const Duration(milliseconds: 80));
    }

    await ask(<ResurfaceRow>[_row('a'), _row('b')]);
    expect(bloc.state.rows.map((r) => r.id), <String>['a', 'b']);
    await ask(<ResurfaceRow>[_row('c'), _row('b'), _row('a')]);
    expect(bloc.state.rows.map((r) => r.id), <String>['a', 'b', 'c']);
    await ask(<ResurfaceRow>[]);
    expect(bloc.state.rows, isEmpty);
    await bloc.close();
  });

  test('"Not related" takes the note away and tells the library', () async {
    answer(<ResurfaceRow>[_row('a'), _row('b')]);
    final bloc = build()..add(const ResurfaceEvent.started('note-a'));
    await Future<void>.delayed(const Duration(milliseconds: 5));
    bloc.add(
      const ResurfaceEvent.edited(
        title: 'T',
        body: 'one two three four five six',
        caret: 3,
      ),
    );
    await Future<void>.delayed(const Duration(milliseconds: 80));
    bloc.add(const ResurfaceEvent.dismissed('a'));
    await Future<void>.delayed(const Duration(milliseconds: 5));
    expect(bloc.state.rows.map((r) => r.id), <String>['b']);
    verify(() => discovery.dismissRelated('note-a', 'a')).called(1);
    await bloc.close();
  });

  g.Glados2(
    g.any.list(g.any.letterOrDigits),
    g.any.list(g.any.letterOrDigits),
  ).test(
    'merging keeps at most three, no duplicates, the old order first',
    (shownIds, foundIds) {
      final shown = <ResurfaceRow>[
        for (final id in shownIds.toSet().take(3)) _row(id),
      ];
      final found = <ResurfaceRow>[for (final id in foundIds.toSet()) _row(id)];
      final merged = mergeResurfaced(shown, found);
      final ids = merged.map((r) => r.id).toList();
      expect(ids.length, lessThanOrEqualTo(resurfaceShown));
      expect(ids.toSet().length, ids.length);
      // Only notes that were found are kept.
      expect(ids.every(foundIds.toSet().contains), isTrue);
      // The ones that stayed keep the order they were shown in.
      final stayed = shown.map((r) => r.id).where(ids.contains).toList();
      expect(ids.take(stayed.length).toList(), stayed);
    },
  );
}
