// Kanso — dictation_scope_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/dictation/models/data/dictation_status.dart';
import 'package:kanso/dictation/models/repos/impl/plain_dictation_text_rules.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/views/widgets/dictation_scope.dart';
import 'package:kanso/editor_extras/models/data/editor_extras.dart';
import 'package:kanso/editor_extras/views/widgets/editor_extras_scope.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/pump.dart';
import '../../helpers/fake_text_target.dart';
import '../../helpers/mock_dictation_repository.dart';
import '../../helpers/seeded_dictation_bloc.dart';

void main() {
  late SpokenRepository spoken;

  setUp(() => spoken = SpokenRepository());
  tearDown(() => spoken.close());

  Future<EditorExtras> pump(
    WidgetTester tester,
    DictationState seed,
  ) async {
    final bloc = SeededDictationBloc(seed, spoken);
    closeBlocLater(bloc);
    late EditorExtras extras;
    await pumpKanso(
      tester,
      DictationScope(
        bloc: bloc,
        rules: PlainDictationTextRules(),
        child: Builder(
          builder: (context) {
            extras = EditorExtrasScope.of(context);
            return const SizedBox.expand();
          },
        ),
      ),
    );
    return extras;
  }

  testWidgets('puts the dictation key in reach of the editor', (tester) async {
    final extras = await pump(tester, const DictationState());
    final keys = extras.accessoryKeys(FakeTextTarget('x'));
    expect(keys.single.id, 'dictation');
    expect(keys.single.semanticLabel, 'Dictate');
  });

  testWidgets('puts its key after the others', (tester) async {
    final bloc = SeededDictationBloc(const DictationState(), spoken);
    closeBlocLater(bloc);
    late EditorExtras extras;
    await pumpKanso(
      tester,
      DictationScope(
        bloc: bloc,
        rules: PlainDictationTextRules(),
        others: const NoEditorExtras(),
        child: Builder(
          builder: (context) {
            extras = EditorExtrasScope.of(context);
            return const SizedBox.expand();
          },
        ),
      ),
    );
    expect(extras.accessoryKeys(FakeTextTarget('x')), hasLength(1));
  });

  testWidgets('shows the pill with Stop while it listens', (tester) async {
    await pump(
      tester,
      const DictationState(
        status: DictationStatus.listening,
        showOverlay: true,
      ),
    );
    expect(find.text('Listening'), findsOneWidget);
    await tester.tap(find.text('Stop'));
    await tester.pump();
    verify(spoken.repository.stop).called(1);
  });

  testWidgets('shows nothing while it is idle', (tester) async {
    await pump(tester, const DictationState());
    expect(find.text('Listening'), findsNothing);
    expect(find.text('Stop'), findsNothing);
  });

  testWidgets('keeps voice-note dictation in its own page', (tester) async {
    await pump(
      tester,
      const DictationState(status: DictationStatus.listening),
    );
    expect(find.text('Listening'), findsNothing);
    expect(find.text('Stop'), findsNothing);
  });

  testWidgets('tells a refused microphone where to allow it', (tester) async {
    await pump(
      tester,
      const DictationState(
        status: DictationStatus.denied,
        showOverlay: true,
      ),
    );
    await tester.tap(find.text('Open Settings'));
    await tester.pump();
    verify(spoken.repository.openSettings).called(1);
  });
}
