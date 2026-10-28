// Kanso — ask_library_row_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/ask/views/widgets/ask_library_row.dart';
import 'package:kanso/core/views/widgets/kanso_sidebar_row.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';

void main() {
  group('AskLibraryRow', () {
    testWidgets('is not there while AI is off', (tester) async {
      final status = await startedAiStatus();
      await pumpRouted(
        tester,
        BlocProvider<AiStatusBloc>.value(
          value: status,
          child: const AskLibraryRow(),
        ),
      );
      expect(find.byType(KansoSidebarRow), findsNothing);
      expect(find.text('Ask your notes'), findsNothing);
    });

    testWidgets('opens Ask while AI is on, and goes when it is turned off', (
      tester,
    ) async {
      final ai = FixtureAiRepository(settings: aiSettings(enabled: true));
      addTearDown(ai.close);
      final status = await startedAiStatus(repository: ai);
      final router = await pumpRouted(
        tester,
        BlocProvider<AiStatusBloc>.value(
          value: status,
          child: const AskLibraryRow(),
        ),
      );
      expect(find.text('Ask your notes'), findsOneWidget);
      await tester.tap(find.text('Ask your notes'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/ask');

      router.pop();
      await tester.pumpAndSettle();
      ai.changes.add(aiSettings());
      await tester.pumpAndSettle();
      expect(find.text('Ask your notes'), findsNothing);
    });
  });
}
