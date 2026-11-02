// Kanso — triage_pager_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/triage_types.dart';
import 'package:kanso/triage/models/data/triage_card_state.dart';
import 'package:kanso/triage/state/triage/triage_bloc.dart';
import 'package:kanso/triage/views/widgets/triage_pager.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/kanso_test_app.dart';

class _MockTriageBloc extends MockBloc<TriageEvent, TriageState>
    implements TriageBloc {}

TriageChip _chip(String value) => TriageChip(
  field: TriageField.tag,
  value: value,
  label: value,
  score: 1,
  because: const <TriageNote>[],
  ai: false,
  isNew: false,
);

TriageCardState _card(
  String id, {
  bool accepted = false,
  bool changed = false,
}) => TriageCardState(
  noteId: id,
  rev: 'r',
  title: 'Note $id',
  snippet: 'About $id',
  updatedMs: 0,
  chips: <TriageChipState>[
    TriageChipState(
      chip: _chip('tag-$id'),
      mark: accepted ? ChipMark.accepted : ChipMark.untouched,
    ),
  ],
  changed: changed,
);

void main() {
  late _MockTriageBloc bloc;

  setUpAll(() {
    registerFallbackValue(const TriageEvent.started());
  });

  setUp(() {
    bloc = _MockTriageBloc();
    when(() => bloc.state).thenReturn(const TriageState());
  });

  Future<void> pump(WidgetTester tester, List<TriageCardState> cards) async {
    await tester.pumpWidget(
      KansoTestApp(
        home: BlocProvider<TriageBloc>.value(
          value: bloc,
          child: SizedBox(
            width: 400,
            height: 800,
            child: TriagePager(cards: cards),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('one card fills the pager, the first one', (tester) async {
    await pump(tester, <TriageCardState>[_card('a'), _card('b')]);
    expect(find.text('Note a'), findsOneWidget);
    expect(find.text('Note b'), findsNothing);
    expect(find.byType(PageView), findsOneWidget);
  });

  testWidgets('a swipe left skips the card', (tester) async {
    await pump(tester, <TriageCardState>[_card('a'), _card('b')]);
    await tester.drag(find.text('Note a'), const Offset(-200, 0));
    await tester.pumpAndSettle();
    verify(() => bloc.add(const TriageEvent.cardSkipped('a'))).called(1);
    verifyNever(() => bloc.add(const TriageEvent.cardFiled('a')));
  });

  testWidgets('a swipe right files a card with something accepted', (
    tester,
  ) async {
    await pump(tester, <TriageCardState>[_card('a', accepted: true)]);
    await tester.drag(find.text('Note a'), const Offset(200, 0));
    await tester.pumpAndSettle();
    verify(() => bloc.add(const TriageEvent.cardFiled('a'))).called(1);
    verifyNever(() => bloc.add(const TriageEvent.cardSkipped('a')));
  });

  testWidgets('a swipe right does nothing while nothing is accepted', (
    tester,
  ) async {
    await pump(tester, <TriageCardState>[_card('a')]);
    await tester.drag(find.text('Note a'), const Offset(200, 0));
    await tester.pumpAndSettle();
    verifyNever(() => bloc.add(any()));
    expect(find.text('Note a'), findsOneWidget);
  });

  testWidgets('a swipe right does nothing for a card whose note changed', (
    tester,
  ) async {
    await pump(tester, <TriageCardState>[
      _card('a', accepted: true, changed: true),
    ]);
    await tester.drag(find.text('Note a'), const Offset(200, 0));
    await tester.pumpAndSettle();
    verifyNever(() => bloc.add(any()));
  });

  testWidgets('a short drag springs back and acts on nothing', (tester) async {
    await pump(tester, <TriageCardState>[_card('a', accepted: true)]);
    await tester.drag(find.text('Note a'), const Offset(40, 0));
    await tester.pumpAndSettle();
    verifyNever(() => bloc.add(any()));
  });

  testWidgets('the next card shows once the first leaves the list', (
    tester,
  ) async {
    await pump(tester, <TriageCardState>[_card('a'), _card('b')]);
    await pump(tester, <TriageCardState>[_card('b')]);
    expect(find.text('Note a'), findsNothing);
    expect(find.text('Note b'), findsOneWidget);
  });
}
