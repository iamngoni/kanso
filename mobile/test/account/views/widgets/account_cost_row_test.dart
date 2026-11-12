// Kanso — account_cost_row_test.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/views/widgets/account_cost_row.dart';

import '../../../helpers/fixture_account_settings_repository.dart';
import '../../../helpers/pump.dart';

void main() {
  group('AccountCostRow', () {
    testWidgets('shows the library’s words and opens the Price page', (
      tester,
    ) async {
      final opened = <Uri>[];
      await pumpKanso(
        tester,
        AccountCostRow(
          cost: sampleCost,
          onOpen: (uri) async => opened.add(uri),
        ),
      );
      expect(find.text('What Kanso costs'), findsOneWidget);
      expect(find.textContaining('Free. No subscription.'), findsOneWidget);
      expect(find.textContaining('23 MiB of 100 MiB notes'), findsOneWidget);
      await tester.tap(find.text('Learn more'));
      await tester.pump();
      expect(opened, <Uri>[Uri.parse('https://kansoapp.app/price')]);
    });
  });
}
