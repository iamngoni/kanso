// Kanso — rename_speakers_sheet_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording_speakers_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/core/views/widgets/kanso_text_field.dart';
import 'package:kanso/recording_speakers/views/widgets/rename_speakers_sheet.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/golden.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/pump_routed.dart';

const List<SpeakerNameView> _names = <SpeakerNameView>[
  SpeakerNameView(name: 'You', paragraphs: 12),
  SpeakerNameView(name: 'Speaker 1', paragraphs: 31),
  SpeakerNameView(name: 'Speaker 2', paragraphs: 1),
];

void main() {
  late MockRecordingSpeakersRepository repository;

  setUpAll(() {
    registerFallbackValue(<SpeakerChange>[]);
    registerFallbackValue(const UndoNoticeEvent.shown(0, ''));
  });

  setUp(() {
    repository = MockRecordingSpeakersRepository();
    when(() => repository.names(any())).thenAnswer(
      (_) async => const Right<ApplicationError, List<SpeakerNameView>>(_names),
    );
  });

  Widget sheet({
    List<SpeakerNameView>? names,
    UndoNoticeBloc? undo,
  }) => Builder(
    builder: (context) => Material(
      color: context.colors.app,
      child: RenameSpeakersSheet(
        noteId: 'note:rec',
        repository: repository,
        undoNotice: undo,
        initialNames: names,
      ),
    ),
  );

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('Rename writes only the changed names and offers Undo', (
    tester,
  ) async {
    final undo = MockUndoNoticeBloc();
    when(() => undo.state).thenReturn(const UndoNoticeState());
    when(() => repository.rename(any(), any())).thenAnswer(
      (_) async => const Right<ApplicationError, SpeakersRenamed>(
        SpeakersRenamed(undo: 9),
      ),
    );
    await pumpRouted(tester, sheet(undo: undo));
    await tester.pumpAndSettle();
    expect(find.text('Speaker 1 · 31 paragraphs'), findsOneWidget);
    expect(find.text('Speaker 2 · 1 paragraph'), findsOneWidget);
    await tester.enterText(
      find.byType(KansoTextField).at(1),
      'Priya',
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Rename'));
    await tester.pumpAndSettle();
    final call = verify(() => repository.rename('note:rec', captureAny()))
      ..called(1);
    expect(call.captured.single, <SpeakerChange>[
      const SpeakerChange(from: 'Speaker 1', to: 'Priya'),
    ]);
    verify(() => undo.add(any())).called(1);
  });

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('changing nothing writes nothing', (tester) async {
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    await tester.tap(find.text('Rename'));
    await tester.pumpAndSettle();
    verifyNever(() => repository.rename(any(), any()));
  });

  // no-prop-test: a note with nobody to rename.
  testWidgets('a note with nobody to rename says what is needed', (
    tester,
  ) async {
    when(() => repository.names(any())).thenAnswer(
      (_) async => const Right<ApplicationError, List<SpeakerNameView>>(
        <SpeakerNameView>[],
      ),
    );
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    expect(find.textContaining('who spoke'), findsOneWidget);
    expect(find.text('Rename'), findsNothing);
  });

  // no-prop-test: a refused write.
  testWidgets('a refused write says why and changes nothing', (tester) async {
    when(() => repository.rename(any(), any())).thenAnswer(
      (_) async => const Left<ApplicationError, SpeakersRenamed>(
        ApplicationError.storage(message: 'The note changed'),
      ),
    );
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(KansoTextField).first, 'Me');
    await tester.tap(find.text('Rename'));
    await tester.pumpAndSettle();
    expect(find.textContaining('The note changed'), findsOneWidget);
    expect(find.text('Rename'), findsNothing);
  });

  for (final brightness in Brightness.values) {
    // no-prop-test: pixels.
    testWidgets('the sheet, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        sheet(names: _names),
        name: 'phone_rename_speakers_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
