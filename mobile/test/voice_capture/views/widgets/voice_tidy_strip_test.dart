// Kanso — voice_tidy_strip_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/voice_capture_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/voice_capture/views/widgets/voice_tidy_strip.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';

void main() {
  late MockVoiceCaptureRepository voice;

  setUp(() => voice = MockVoiceCaptureRepository());

  void info({required bool canTidy, String tidy = 'none'}) =>
      when(() => voice.noteInfo(any())).thenAnswer(
        (_) async => Right<ApplicationError, VoiceNoteInfo?>(
          VoiceNoteInfo(tidy: tidy, durationMs: 5000, canTidy: canTidy),
        ),
      );

  Future<void> pump(WidgetTester tester) async {
    await pumpRouted(
      tester,
      Scaffold(
        body: VoiceTidyStrip(noteId: 'note:cap-1', repository: voice),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('a note that cannot be tidied shows nothing', (tester) async {
    info(canTidy: false);
    await pump(tester);
    expect(find.text('Tidy'), findsNothing);
  });

  testWidgets('a voice note offers Tidy, and tidying reads it again', (
    tester,
  ) async {
    info(canTidy: true);
    when(() => voice.tidy(any())).thenAnswer((_) async {
      info(canTidy: false, tidy: 'done');
      return const Right<ApplicationError, TidyOutcome>(
        TidyOutcome(insertedAbove: false, dropped: 0),
      );
    });
    await pump(tester);
    expect(find.text('Tidy'), findsOneWidget);
    await tester.tap(find.text('Tidy'));
    await tester.pumpAndSettle();
    verify(() => voice.tidy('note:cap-1')).called(1);
    expect(find.text('Tidy'), findsNothing);
  });

  testWidgets('a failed tidy says why and offers another try', (tester) async {
    info(canTidy: true);
    when(() => voice.tidy(any())).thenAnswer(
      (_) async => const Left<ApplicationError, TidyOutcome>(
        ApplicationError.storage(message: 'The model is busy'),
      ),
    );
    await pump(tester);
    await tester.tap(find.text('Tidy'));
    await tester.pumpAndSettle();
    expect(find.textContaining('busy'), findsOneWidget);
    expect(find.text('Try again'), findsOneWidget);
  });
}
