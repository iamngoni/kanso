// Kanso — resurface_pill_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart' hide LockState;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/note/state/resurface/resurface_bloc.dart';
import 'package:kanso/note/views/widgets/related_notes_sheet.dart';
import 'package:kanso/note/views/widgets/resurface_pill.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_discovery_repository.dart';
import '../../../helpers/pump_routed.dart';

class _MockPreferences extends Mock implements PreferencesRepository {}

class _Discovery extends FixtureDiscoveryRepository {
  _Discovery({this.related = true, this.rows = const <ResurfaceRow>[]});

  final bool related;
  final List<ResurfaceRow> rows;
  List<OnThisDayRow> day = const <OnThisDayRow>[];

  @override
  Future<Either<ApplicationError, bool>> relatedEnabled() async =>
      Right<ApplicationError, bool>(related);

  @override
  Future<Either<ApplicationError, ResurfaceResult>> resurface({
    required String id,
    required String title,
    required String body,
    required int caret,
    required String closeness,
    required int utcOffsetSecs,
  }) async => Right<ApplicationError, ResurfaceResult>(
    ResurfaceResult.found(rows: rows),
  );

  @override
  Future<Either<ApplicationError, List<OnThisDayRow>>> onThisDay(
    String id, {
    required int utcOffsetSecs,
  }) async => Right<ApplicationError, List<OnThisDayRow>>(day);

  @override
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  }) async => const Right<ApplicationError, List<RelatedRow>>(<RelatedRow>[]);
}

ResurfaceRow _row(String id, String title) => ResurfaceRow(
  id: id,
  title: title,
  book: 'Home',
  when: 'March 2024',
  snippet: '…the pressure valve…',
);

ResurfaceBloc _bloc(_Discovery discovery, {Preferences? preferences}) {
  final repository = _MockPreferences();
  when(
    repository.load,
  ).thenAnswer((_) async => right(preferences ?? const Preferences()));
  return ResurfaceBloc(discovery, repository)
    ..settle = const Duration(milliseconds: 10)
    ..utcOffsetSecs = () => 0;
}

Widget _provide(ResurfaceBloc bloc, Widget child) =>
    BlocProvider<ResurfaceBloc>.value(
      value: bloc,
      child: Scaffold(body: child),
    );

void main() {
  testWidgets('reading mode shows the pill with no delay between events', (
    tester,
  ) async {
    final bloc = _bloc(_Discovery(rows: <ResurfaceRow>[_row('a', 'Boiler')]))
      ..add(const ResurfaceEvent.started('note-a'));
    await pumpRouted(
      tester,
      _provide(
        bloc,
        const ResurfaceReading(
          noteId: 'note-a',
          title: 'T',
          body: 'one two three four five six seven',
          child: SizedBox.expand(),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('1 related'), findsOneWidget);
  });

  testWidgets('"On this day" alone shows the pill, with paragraphs off', (
    tester,
  ) async {
    final discovery = _Discovery(related: false)
      ..day = const <OnThisDayRow>[
        OnThisDayRow(id: 'old', title: 'Walk', book: 'Journal', ago: '2 years'),
      ];
    final bloc = _bloc(discovery)..add(const ResurfaceEvent.started('note-a'));
    await pumpRouted(
      tester,
      _provide(bloc, const ResurfacePill(noteId: 'note-a')),
    );
    await tester.pump(const Duration(milliseconds: 100));
    expect(bloc.state.enabled, isFalse);
    expect(find.text('1 related'), findsOneWidget);
  });

  testWidgets('the pill row takes no room while there is no pill', (
    tester,
  ) async {
    final bloc = _bloc(_Discovery())
      ..add(const ResurfaceEvent.started('note-a'));
    await pumpRouted(
      tester,
      _provide(bloc, const ResurfacePillRow(noteId: 'note-a')),
    );
    await tester.pump(const Duration(milliseconds: 100));
    expect(tester.getSize(find.byType(ResurfacePillRow)).height, 0);
  });

  testWidgets('a note with no title has no Link, a titled one has', (
    tester,
  ) async {
    final bloc =
        _bloc(
            _Discovery(
              rows: <ResurfaceRow>[_row('a', ''), _row('b', 'Boiler')],
            ),
          )
          ..add(const ResurfaceEvent.started('note-a'))
          ..add(
            const ResurfaceEvent.edited(
              title: 'T',
              body: 'one two three four five six seven',
              caret: 3,
            ),
          );
    await pumpRouted(
      tester,
      _provide(
        bloc,
        RelatedNotesSheet(
          noteId: 'note-a',
          discovery: _Discovery(),
          resurface: bloc,
          onLink: (_) {},
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump(const Duration(milliseconds: 100));
    expect(bloc.state.rows.length, 2);
    expect(find.text('Link'), findsOneWidget);
  });
}
