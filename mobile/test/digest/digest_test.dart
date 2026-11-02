// Kanso — digest_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, IntAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/digest/views/pages/open_loops_page.dart';
import 'package:kanso/digest/views/widgets/brief_section.dart';
import 'package:kanso/injection.dart';

import '../helpers/fixture_ai_repository.dart';
import '../helpers/fixture_digest_repository.dart';
import '../helpers/fixture_library_repository.dart';
import '../helpers/property.dart';
import '../helpers/pump_routed.dart';

const _mine = OpenLoopItem(
  key: 'k1',
  noteId: 'note:a',
  noteTitle: 'Plans',
  notebookId: 'book:a',
  line: 0,
  kind: OpenLoopKind.mine,
  text: "I'll send the deck to Sam.",
  ageLabel: '3 days ago',
  fromAi: false,
);
const _waiting = OpenLoopItem(
  key: 'k2',
  noteId: 'note:a',
  noteTitle: 'Plans',
  notebookId: 'book:a',
  line: 1,
  kind: OpenLoopKind.waiting,
  text: 'Waiting on Rudo for the invoice.',
  dueLabel: 'Fri 6 Nov',
  ageLabel: 'today',
  fromAi: true,
);

void main() {
  group('stepTime', () {
    Glados2<int, int>(
      any.intInRange(0, 24 * 60),
      any.intInRange(-2000, 2000),
      kansoExploreConfig(),
    ).test('stays a time of day and stepping back undoes it', (start, delta) {
      final (h, m) = stepTime(start ~/ 60, start % 60, delta);
      expect(h, inInclusiveRange(0, 23));
      expect(m, inInclusiveRange(0, 59));
      expect(stepTime(h, m, -delta), (start ~/ 60, start % 60));
    });

    test('wraps around midnight', () {
      expect(stepTime(23, 50, 15), (0, 5));
      expect(stepTime(0, 5, -15), (23, 50));
    });
  });

  group('OpenLoopsPage', () {
    late FixtureDigestRepository digest;
    late FixtureLibraryRepository library;

    setUp(() async {
      await getIt.reset();
      digest = FixtureDigestRepository()
        ..loops = <OpenLoopItem>[_mine, _waiting];
      library = FixtureLibraryRepository();
      getIt
        ..registerSingleton<DigestRepository>(digest)
        ..registerSingleton<AiRepository>(FixtureAiRepository())
        ..registerSingleton<LibraryRepository>(library);
      addTearDown(library.close);
      addTearDown(getIt.reset);
    });

    testWidgets('groups the loops by kind and says how old they are', (
      tester,
    ) async {
      await pumpRouted(tester, const OpenLoopsPage());
      await tester.pumpAndSettle();
      expect(find.text('Open loops'), findsOneWidget);
      expect(find.text('2 open loops'), findsOneWidget);
      expect(find.text('I will'), findsOneWidget);
      expect(find.text('Waiting on'), findsOneWidget);
      expect(find.text("I'll send the deck to Sam."), findsOneWidget);
      expect(find.text('Plans · 3 days ago'), findsOneWidget);
      expect(
        find.text('Plans · today · Fri 6 Nov · Found by AI'),
        findsOneWidget,
      );
    });

    testWidgets('says so when there are none', (tester) async {
      digest.loops = <OpenLoopItem>[];
      await pumpRouted(tester, const OpenLoopsPage());
      await tester.pumpAndSettle();
      expect(find.text('No open loops'), findsOneWidget);
    });
  });
}
