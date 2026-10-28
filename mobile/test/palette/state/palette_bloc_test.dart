// Kanso — palette_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/palette/models/data/palette_status.dart';
import 'package:kanso/palette/models/repos/abs/palette_repository.dart';
import 'package:kanso/palette/state/palette/palette_bloc.dart';

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

const _note = PaletteSection(
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

/// Answers with [_note] for nothing typed, and with what [answers] says for
/// everything else.
class _FakePalette implements PaletteRepository {
  _FakePalette({this.answers = const <String, Object>{}});

  /// What [search] gives per query: a list of sections, or an error.
  final Map<String, Object> answers;
  final List<String> asked = <String>[];

  @override
  Future<Either<ApplicationError, List<PaletteSection>>> search({
    required String query,
    required PaletteContext context,
  }) async {
    asked.add(query);
    final answer = answers[query];
    if (answer is ApplicationError) {
      return Left(answer);
    }
    if (answer is List<PaletteSection>) {
      return Right(answer);
    }
    return const Right(<PaletteSection>[_note]);
  }
}

void main() {
  blocTest<PaletteBloc, PaletteState>(
    'opens asking for nothing typed, and shows what matches',
    build: () => PaletteBloc(_FakePalette()),
    act: (bloc) => bloc.add(const PaletteEvent.started(_context)),
    wait: const Duration(milliseconds: 300),
    expect: () => <Matcher>[
      isA<PaletteState>().having(
        (state) => state.status,
        'status',
        PaletteStatus.loading,
      ),
      isA<PaletteState>()
          .having((state) => state.status, 'status', PaletteStatus.ready)
          .having((state) => state.sections.length, 'sections', 1)
          .having((state) => state.isEmpty, 'isEmpty', false),
    ],
  );

  blocTest<PaletteBloc, PaletteState>(
    'a query replaces what the empty field found',
    build: () => PaletteBloc(
      _FakePalette(
        answers: <String, Object>{
          'pin': const <PaletteSection>[],
        },
      ),
    ),
    act: (bloc) async {
      bloc.add(const PaletteEvent.started(_context));
      await Future<void>.delayed(const Duration(milliseconds: 200));
      bloc.add(const PaletteEvent.queryChanged('pin'));
    },
    wait: const Duration(milliseconds: 400),
    verify: (bloc) {
      expect(bloc.state.query, 'pin');
      expect(bloc.state.status, PaletteStatus.ready);
      expect(bloc.state.sections, isEmpty);
      expect(bloc.state.isEmpty, isTrue);
    },
  );

  blocTest<PaletteBloc, PaletteState>(
    'a failure says why instead of listing',
    build: () => PaletteBloc(
      _FakePalette(
        answers: <String, Object>{
          'tea': const ApplicationError.storage(message: 'Disk error.'),
        },
      ),
    ),
    act: (bloc) => bloc
      ..add(const PaletteEvent.started(_context))
      ..add(const PaletteEvent.queryChanged('tea')),
    wait: const Duration(milliseconds: 400),
    verify: (bloc) {
      expect(bloc.state.status, PaletteStatus.failed);
      expect(bloc.state.problem, 'Disk error.');
      expect(bloc.state.sections, isEmpty);
    },
  );
}
