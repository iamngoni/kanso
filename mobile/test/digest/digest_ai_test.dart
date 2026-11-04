// Kanso — digest_ai_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/digest/state/digest_ai_cubit.dart';
import 'package:kanso/digest/views/pages/open_loops_page.dart';
import 'package:kanso/injection.dart';

import '../helpers/fixture_ai_repository.dart';
import '../helpers/fixture_digest_repository.dart';
import '../helpers/fixture_library_repository.dart';
import '../helpers/pump_routed.dart';

const _sam = LoopProposal(
  noteId: 'note:a',
  noteTitle: 'Plans',
  line: 2,
  text: 'Sam will bring the keys on Friday',
  kind: OpenLoopKind.waiting,
);
const _rent = LoopProposal(
  noteId: 'note:b',
  noteTitle: '',
  line: 0,
  text: 'Check whether the rent went out',
  kind: OpenLoopKind.question,
);

AiSettings _on({bool hasKey = true}) => AiSettings(
  enabled: true,
  provider: 'anthropic',
  host: 'api.anthropic.com',
  hasKey: hasKey,
);

void main() {
  late FixtureDigestRepository digest;
  late FixtureAiRepository ai;
  late FixtureLibraryRepository library;

  setUp(() async {
    await getIt.reset();
    digest = FixtureDigestRepository();
    ai = FixtureAiRepository(settings: _on());
    library = FixtureLibraryRepository();
    getIt
      ..registerSingleton<DigestRepository>(digest)
      ..registerSingleton<AiRepository>(ai)
      ..registerSingleton<LibraryRepository>(library);
    addTearDown(library.close);
    addTearDown(getIt.reset);
  });

  group('DigestAiCubit', () {
    blocTest<DigestAiCubit, DigestAiState>(
      'is not ready while AI is off, or while a provider lacks its key',
      build: () => DigestAiCubit(digest, ai),
      setUp: () => ai.current = _on(hasKey: false),
      act: (cubit) => cubit.load(),
      verify: (cubit) => expect(cubit.state.ready, isFalse),
    );

    blocTest<DigestAiCubit, DigestAiState>(
      'is ready with a key, and a local provider needs none',
      build: () => DigestAiCubit(digest, ai),
      setUp: () => ai.current = _on().copyWith(
        provider: 'ollama',
        hasKey: false,
      ),
      act: (cubit) => cubit.load(),
      verify: (cubit) => expect(cubit.state.ready, isTrue),
    );

    blocTest<DigestAiCubit, DigestAiState>(
      'finds proposals, keeps only the ticked ones and nothing before',
      build: () => DigestAiCubit(digest, ai),
      setUp: () => digest.scan = const LoopScan(
        proposals: <LoopProposal>[_sam, _rent],
        droppedText: '1 suggestion no longer matches its note',
      ),
      act: (cubit) async {
        await cubit.load();
        await cubit.findLoops();
        expect(digest.kept, isEmpty, reason: 'nothing is kept unasked');
        cubit
          ..toggle(1)
          ..toggle(0)
          ..toggle(0);
        await cubit.keepSelected();
      },
      verify: (cubit) {
        expect(digest.kept, <List<LoopProposal>>[
          <LoopProposal>[_rent],
        ]);
        expect(cubit.state.found, isNull);
        expect(cubit.state.kept, 1);
      },
    );

    blocTest<DigestAiCubit, DigestAiState>(
      'says so when it finds nothing, keeping the dropped line',
      build: () => DigestAiCubit(digest, ai),
      setUp: () => digest.scan = const LoopScan(
        proposals: <LoopProposal>[],
        droppedText: '2 suggestions no longer match their notes',
      ),
      act: (cubit) async {
        await cubit.load();
        await cubit.findLoops();
      },
      verify: (cubit) {
        expect(cubit.state.found, isNull);
        expect(
          cubit.state.empty?.droppedText,
          '2 suggestions no longer match their notes',
        );
      },
    );

    blocTest<DigestAiCubit, DigestAiState>(
      'does nothing at all while AI is not ready',
      build: () => DigestAiCubit(digest, ai),
      act: (cubit) async {
        await cubit.findLoops();
      },
      expect: () => <DigestAiState>[],
    );
  });

  group('the pages', () {
    testWidgets('offer Find more open loops only when AI is set up', (
      tester,
    ) async {
      ai.current = const AiSettings();
      digest.loops = <OpenLoopItem>[];
      await pumpRouted(tester, const OpenLoopsPage());
      await tester.pumpAndSettle();
      expect(find.text('Find more open loops…'), findsNothing);
    });

    testWidgets('ask, show the proposals, and keep the ticked', (
      tester,
    ) async {
      digest
        ..loops = <OpenLoopItem>[]
        ..scan = const LoopScan(
          proposals: <LoopProposal>[_sam, _rent],
          droppedText: '1 suggestion no longer matches its note',
        );
      await pumpRouted(tester, const OpenLoopsPage());
      await tester.pumpAndSettle();
      await tester.tap(find.text('Find more open loops…'));
      await tester.pumpAndSettle();
      expect(find.textContaining('api.anthropic.com'), findsOneWidget);
      await tester.tap(find.text('Find loops'));
      await tester.pumpAndSettle();
      expect(find.text('Open loops found'), findsOneWidget);
      expect(
        find.textContaining('1 suggestion no longer matches its note'),
        findsOneWidget,
      );
      await tester.tap(find.textContaining(_sam.text));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Keep selected'));
      await tester.pumpAndSettle();
      expect(digest.kept.single.single.text, _sam.text);
      expect(find.text('Kept 1 open loop.'), findsOneWidget);
    });

    testWidgets('closing the sheet keeps nothing', (tester) async {
      digest
        ..loops = <OpenLoopItem>[]
        ..scan = const LoopScan(proposals: <LoopProposal>[_sam]);
      await pumpRouted(tester, const OpenLoopsPage());
      await tester.pumpAndSettle();
      await tester.tap(find.text('Find more open loops…'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Find loops'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(digest.kept, isEmpty);
    });
  });
}
