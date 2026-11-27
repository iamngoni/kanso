// Kanso — new_notebook_sheet_goldens_test.dart
// Created 2026-11-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet.dart';
import 'package:kanso/core/views/widgets/kanso_scrim.dart';
import 'package:kanso/library/views/widgets/new_notebook_sheet.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/register_page_blocs.dart';

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(() => unawaited(library.close()));
    registerPageBlocs(library);
  });

  // The sheet as the route shows it: over its veil, at the foot of the
  // screen, as wide as a phone or, on a tablet, 560 points.
  Widget shown() => const Scaffold(
    resizeToAvoidBottomInset: false,
    body: Stack(
      children: <Widget>[
        Positioned.fill(child: KansoScrim()),
        KansoBottomSheet(child: NewNotebookSheet()),
      ],
    ),
  );

  // Product development chosen, with a name typed.
  Future<void> chooseProduct(WidgetTester tester) async {
    await tester.enterText(find.byType(EditableText), 'Kanso');
    await tester.pumpAndSettle();
    await tester.tap(find.text('Product development'));
    await tester.pumpAndSettle();
  }

  group('NewNotebookSheet pictures', () {
    for (final brightness in Brightness.values) {
      testWidgets('on a phone, ${brightness.name}', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          shown(),
          name: 'phone_new_notebook_product_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
          interact: chooseProduct,
        );
      });

      testWidgets('on a tablet, ${brightness.name}', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          shown(),
          name: 'tablet_new_notebook_product_${brightness.name}',
          size: tabletPortraitSize,
          brightness: brightness,
          interact: chooseProduct,
        );
      });
    }
  });
}
