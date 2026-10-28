// Kanso — daily_note_page_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/state/daily_note/daily_note_bloc.dart';
import 'package:kanso/capture/views/pages/daily_note_page.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/injection.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/mock_repositories.dart';

/// A 10 inch Android tablet held sideways: wide enough for two panes.
const Size androidTabletSize = Size(962, 601);

/// The sizes the page is checked at, with the panes each has room for.
final Map<String, (Size, LayoutClass)> _sizes = <String, (Size, LayoutClass)>{
  'iphone': (phoneSize, LayoutClass.single),
  'ipad_portrait': (tabletPortraitSize, LayoutClass.double),
  'ipad_landscape': (tabletLandscapeSize, LayoutClass.triple),
  'android_tablet': (androidTabletSize, LayoutClass.double),
};

void main() {
  late MockCaptureRepository capture;

  setUpAll(() => registerFallbackValue(DateTime(2026)));

  setUp(() {
    capture = MockCaptureRepository();
    getIt.registerFactory<DailyNoteBloc>(() => DailyNoteBloc(capture));
    addTearDown(() => unawaited(getIt.reset()));
  });

  group("today's note", () {
    for (final entry in _sizes.entries) {
      final (size, layout) = entry.value;
      for (final brightness in Brightness.values) {
        final name = '${entry.key}_${brightness.name}';

        testWidgets('while it opens, $name', (tester) async {
          expect(KansoLayout.classFor(size), layout);
          when(() => capture.todayNote(any())).thenAnswer(
            (_) => Completer<Either<ApplicationError, String>>().future,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            const DailyNotePage(),
            name: 'daily_note_loading_$name',
            size: size,
            brightness: brightness,
          );
        });

        testWidgets('after a failure, $name', (tester) async {
          expect(KansoLayout.classFor(size), layout);
          when(() => capture.todayNote(any())).thenAnswer(
            (_) async => const Left<ApplicationError, String>(
              ApplicationError.storage(message: 'Busy.'),
            ),
          );
          await expectGoldenOnBothPlatforms(
            tester,
            const DailyNotePage(),
            name: 'daily_note_failed_$name',
            size: size,
            brightness: brightness,
          );
        });
      }
    }
  });
}
