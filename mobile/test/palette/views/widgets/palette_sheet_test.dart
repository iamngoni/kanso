// Kanso — palette_sheet_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet_route.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/palette/models/repos/abs/palette_repository.dart';
import 'package:kanso/palette/state/palette/palette_bloc.dart';
import 'package:kanso/palette/views/widgets/palette_sheet.dart';

const _context = PaletteContext(
  signedIn: false,
  sessionHeld: false,
  aiOn: false,
  settingsOpen: false,
  imageLine: false,
  audioLine: false,
  sharedNote: false,
  handlers: <String>[],
  overdueTasks: 0,
);

const _commands = PaletteSection(
  label: 'Note actions',
  entries: <PaletteEntry>[
    PaletteEntry(
      target: PaletteTarget.command(id: 'new-note'),
      label: 'New note',
      icon: 'file-plus',
      checked: false,
      depth: 0,
    ),
  ],
);

const _places = PaletteSection(
  label: 'Go to',
  entries: <PaletteEntry>[
    PaletteEntry(
      target: PaletteTarget.place(place: PalettePlace.pinned()),
      label: 'Pinned',
      icon: 'pin',
      hint: '12 notes',
      checked: true,
      depth: 0,
    ),
  ],
);

/// Answers with the sections it was given.
class _FakePalette implements PaletteRepository {
  _FakePalette(this.sections);

  final List<PaletteSection> sections;

  @override
  Future<Either<ApplicationError, List<PaletteSection>>> search({
    required String query,
    required PaletteContext context,
  }) async => Right(sections);
}

/// A bloc for [sections], closed when the test is over.
///
/// It is not awaited: the palette's debounce holds a timer that only runs as
/// the test pumps, so closing a bloc with one still held would wait forever.
PaletteBloc _blocFor(List<PaletteSection> sections) {
  final bloc = PaletteBloc(_FakePalette(sections))
    ..add(const PaletteEvent.started(_context));
  addTearDown(() {
    unawaited(bloc.close());
  });
  return bloc;
}

Widget _app(Widget child) => MaterialApp(
  theme: KansoTheme.light,
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  home: child,
);

void main() {
  testWidgets('lists the sections and their rows', (tester) async {
    final bloc = _blocFor(const <PaletteSection>[_commands, _places]);
    await tester.pumpWidget(
      _app(
        BlocProvider<PaletteBloc>.value(
          value: bloc,
          child: Scaffold(body: PaletteSheet(onSelected: (_) {})),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.text('Note actions'), findsOneWidget);
    expect(find.text('New note'), findsOneWidget);
    expect(find.text('Go to'), findsOneWidget);
    expect(find.text('Pinned'), findsOneWidget);
    expect(find.text('12 notes'), findsOneWidget);
    expect(find.byType(KansoSearchInput), findsOneWidget);
  });

  testWidgets('reports the row chosen and closes over it', (tester) async {
    final bloc = _blocFor(const <PaletteSection>[_commands, _places]);
    final chosen = <PaletteTarget>[];
    await tester.pumpWidget(
      _app(
        Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: TextButton(
                onPressed: () => showKansoBottomSheet<void>(
                  context,
                  builder: (_) => BlocProvider<PaletteBloc>.value(
                    value: bloc,
                    child: PaletteSheet(onSelected: chosen.add),
                  ),
                ),
                child: const Text('open'),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.text('New note'), findsOneWidget);
    await tester.tap(find.text('New note'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(chosen.single, const PaletteTarget.command(id: 'new-note'));
    expect(find.text('Note actions'), findsNothing);
  });

  testWidgets('says when nothing matches', (tester) async {
    final bloc = _blocFor(const <PaletteSection>[]);
    await tester.pumpWidget(
      _app(
        BlocProvider<PaletteBloc>.value(
          value: bloc,
          child: Scaffold(body: PaletteSheet(onSelected: (_) {})),
        ),
      ),
    );
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.text('Nothing matches'), findsOneWidget);
  });
}
