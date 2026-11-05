// Kanso — recording_summary_sheet_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording_summary_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/recording_summary/views/widgets/recording_summary_sheet.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/golden.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/pump_routed.dart';

const String _send =
    "Send this recording's transcript (8412 words), with the note's title and "
    'the date, to api.openai.com? The audio is not sent. The transcript is. '
    'api.openai.com handles it under its own retention rules.';

const SummaryOffer _offer = SummaryOffer(
  available: true,
  words: 8412,
  sendLine: _send,
  replaces: false,
);

const SummaryDraftView _draft = SummaryDraftView(
  fileId: 'file:abc123',
  overview: <String>[
    'The beta ships on the 6th.',
    'Onboarding is the open risk.',
  ],
  decisions: <SummaryItemView>[
    SummaryItemView(text: 'Ship the beta on the 6th', moment: 22),
  ],
  actions: <SummaryItemView>[
    SummaryItemView(
      text: 'Send the onboarding draft',
      who: 'Priya',
      when: 'by the 30th',
      moment: 47,
    ),
    SummaryItemView(text: 'Book the review with design', moment: 1340),
  ],
  dropped: 1,
  cut: false,
);

void main() {
  late MockRecordingSummaryRepository repository;

  setUpAll(() {
    registerFallbackValue(_draft);
    registerFallbackValue(<int>[]);
    registerFallbackValue(const UndoNoticeEvent.shown(0, ''));
  });

  setUp(() {
    repository = MockRecordingSummaryRepository();
    when(
      () => repository.offer(any()),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, SummaryOffer>(_offer),
    );
  });

  Widget sheet({
    SummaryPhase? phase,
    SummaryDraftView? draft,
    UndoNoticeBloc? undo,
  }) => Builder(
    builder: (context) => Material(
      color: context.colors.app,
      child: RecordingSummarySheet(
        noteId: 'note:rec',
        repository: repository,
        undoNotice: undo,
        initialPhase: phase,
        initialOffer: phase == null ? null : _offer,
        initialDraft: draft,
      ),
    ),
  );

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('nothing is sent until Send, and only Send asks', (tester) async {
    when(() => repository.summarize(any())).thenAnswer(
      (_) async => const Right<ApplicationError, SummaryDraftView>(_draft),
    );
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    expect(find.textContaining('The audio is not sent.'), findsOneWidget);
    verifyNever(() => repository.summarize(any()));
    await tester.tap(find.text('Send'));
    await tester.pumpAndSettle();
    verify(() => repository.summarize('note:rec')).called(1);
    expect(find.text('Send the onboarding draft'), findsOneWidget);
    expect(find.text('Action items'), findsOneWidget);
  });

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('Apply writes only the ticked items and offers Undo', (
    tester,
  ) async {
    final undo = MockUndoNoticeBloc();
    when(() => undo.state).thenReturn(const UndoNoticeState());
    when(() => repository.apply(any(), any(), any())).thenAnswer(
      (_) async => const Right<ApplicationError, SummaryApplied>(
        SummaryApplied(replaced: false, undo: 7),
      ),
    );
    await pumpRouted(
      tester,
      sheet(phase: SummaryPhase.review, draft: _draft, undo: undo),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Book the review with design'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    final call = verify(
      () => repository.apply('note:rec', any(), captureAny()),
    )..called(1);
    expect(call.captured.single, <int>[0]);
    verify(() => undo.add(any())).called(1);
  });

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('a failure says so, changes nothing and offers another try', (
    tester,
  ) async {
    when(() => repository.summarize(any())).thenAnswer(
      (_) async => const Left<ApplicationError, SummaryDraftView>(
        ApplicationError.storage(message: 'The model is busy'),
      ),
    );
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    await tester.tap(find.text('Send'));
    await tester.pumpAndSettle();
    expect(find.textContaining('The model is busy'), findsNothing);
    expect(find.text('Try again'), findsOneWidget);
    verifyNever(() => repository.apply(any(), any(), any()));
  });

  // no-prop-test: drives the sheet over a mocked repository.
  testWidgets('an Apply that writes nothing keeps the review and says why', (
    tester,
  ) async {
    when(() => repository.apply(any(), any(), any())).thenAnswer(
      (_) async => const Left<ApplicationError, SummaryApplied>(
        ApplicationError.invalid(message: 'This note is kept out of AI.'),
      ),
    );
    await pumpRouted(
      tester,
      sheet(phase: SummaryPhase.review, draft: _draft),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect(
      find.text('Nothing was written. This note is kept out of AI.'),
      findsOneWidget,
    );
    // What was looked over is still there, and Apply can be pressed again.
    expect(find.text('Send the onboarding draft'), findsOneWidget);
    expect(find.text('Apply'), findsOneWidget);
    verifyNever(() => repository.summarize(any()));
  });

  // no-prop-test: a note that cannot be summarised.
  testWidgets('a note that cannot be summarised says what is needed', (
    tester,
  ) async {
    when(() => repository.offer(any())).thenAnswer(
      (_) async => const Right<ApplicationError, SummaryOffer>(
        SummaryOffer(available: false, words: 0, sendLine: '', replaces: false),
      ),
    );
    await pumpRouted(tester, sheet());
    await tester.pumpAndSettle();
    expect(find.textContaining('turn on AI in Settings'), findsOneWidget);
    expect(find.text('Send'), findsNothing);
  });

  for (final brightness in Brightness.values) {
    // no-prop-test: pixels.
    testWidgets('the sheet before sending, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        sheet(phase: SummaryPhase.confirm),
        name: 'phone_recording_summary_send_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    // no-prop-test: pixels.
    testWidgets('the review, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        sheet(phase: SummaryPhase.review, draft: _draft),
        name: 'phone_recording_summary_review_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    // no-prop-test: pixels.
    testWidgets('the review of a cut transcript, ${brightness.name}', (
      tester,
    ) async {
      await expectGoldenOnBothPlatforms(
        tester,
        sheet(
          phase: SummaryPhase.review,
          draft: const SummaryDraftView(
            fileId: 'file:abc123',
            overview: <String>['The beta ships on the 6th.'],
            decisions: <SummaryItemView>[],
            actions: <SummaryItemView>[
              SummaryItemView(text: 'Send the onboarding draft', moment: 47),
            ],
            dropped: 0,
            cut: true,
          ),
        ),
        name: 'phone_recording_summary_review_cut_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
