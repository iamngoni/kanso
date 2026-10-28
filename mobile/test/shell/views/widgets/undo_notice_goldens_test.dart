// Kanso — undo_notice_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/kanso_viewport.dart';

void main() {
  // The notice afloat at the foot of the screen after a swipe to the
  // trash, over the list (and over the note beside it on a tablet).
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final viewport in viewports) {
    for (final brightness in Brightness.values) {
      testWidgets('the Undo notice, ${viewport.slug}, ${brightness.name}', (
        tester,
      ) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'undo_notice_${viewport.slug}_${brightness.name}',
          viewport: viewport,
          brightness: brightness,
          library: () =>
              FixtureLibraryRepository()
                ..changeResult = const Right<ApplicationError, Changed>(
                  Changed(
                    count: 1,
                    undo: 1,
                    message: 'Moved 1 note to the trash.',
                  ),
                ),
          interact: (tester, app) async {
            await tester.drag(
              find.text('Sourdough starter'),
              const Offset(-400, 0),
            );
            await tester.pumpAndSettle();
          },
        );
      });
    }
  }
}
