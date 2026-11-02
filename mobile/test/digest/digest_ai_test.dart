// Kanso — digest_ai_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/digest/state/digest_ai_cubit.dart';
import 'package:kanso/digest/views/pages/brief_page.dart';
import 'package:kanso/digest/views/pages/open_loops_page.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/views/widgets/morning_brief_card.dart';

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
        await cubit.summarise();
      },
      expect: () => <DigestAiState>[],
    );

    blocTest<DigestAiCubit, DigestAiState>(
      'writes a summary',
      build: () => DigestAiCubit(digest, ai),
      act: (cubit) async {
        await cubit.load();
        await cubit.summarise();
      },
      verify: (cubit) => expect(cubit.state.summary?.text, 'A calm day.'),
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

    testWidgets('the brief can be summarised, with the model’s words', (
      tester,
    ) async {
      digest.briefView = const BriefView(
        dateLabel: 'Monday 2 November',
        due: <TaskItem>[],
        loops: <OpenLoopItem>[],
        loopsTotal: 2,
        laterToday: <BriefReminder>[],
        relevant: <BriefNote>[],
        wroteYesterday: 0,
        finishedYesterday: 0,
        isEmpty: false,
      );
      await pumpRouted(tester, const BriefPage());
      await tester.pumpAndSettle();
      await tester.tap(find.text('Summarise'));
      await tester.pumpAndSettle();
      expect(find.text('A calm day.'), findsOneWidget);
      expect(find.text('Summary by AI'), findsOneWidget);
    });
  });

  group('MorningBriefCard', () {
    Future<void> pumpCard(WidgetTester tester, String noteId) =>
        withClock(Clock.fixed(DateTime(2026, 11, 2, 9)), () async {
          await pumpRouted(tester, MorningBriefCard(noteId: noteId));
          await tester.pumpAndSettle();
        });

    testWidgets('shows over today’s note', (tester) async {
      await pumpCard(tester, 'note:daily-2026-11-02');
      expect(find.text('Your morning brief'), findsOneWidget);
      expect(find.text('Nothing due, no open loops'), findsOneWidget);
    });

    testWidgets('shows nothing over another day or another note', (
      tester,
    ) async {
      await pumpCard(tester, 'note:daily-2026-11-01');
      expect(find.text('Your morning brief'), findsNothing);
      await pumpCard(tester, 'note:plans');
      expect(find.text('Your morning brief'), findsNothing);
    });

    testWidgets('shows nothing with the brief turned off', (tester) async {
      digest.saved = const BriefPrefs(
        loops: true,
        brief: false,
        briefHour: 8,
        briefMinute: 0,
        briefWeekdaysOnly: false,
        review: true,
        reviewDay: 6,
        reviewHour: 18,
        reviewMinute: 0,
        notifyBrief: true,
        notifyReview: true,
        markDoneInNote: false,
      );
      await pumpCard(tester, 'note:daily-2026-11-02');
      expect(find.text('Your morning brief'), findsNothing);
    });

    testWidgets('fits the narrowest pane', (tester) async {
      await withClock(Clock.fixed(DateTime(2026, 11, 2, 9)), () async {
        await pumpRouted(
          tester,
          const MorningBriefCard(noteId: 'note:daily-2026-11-02'),
          size: const Size(180, 600),
        );
        await tester.pumpAndSettle();
      });
      expect(tester.takeException(), isNull);
    });
  });
}
