// Kanso — meaning_search_results_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/search/views/widgets/meaning_search_results.dart';

import '../../../helpers/golden.dart';

const List<RelatedRow> _rows = <RelatedRow>[
  RelatedRow(
    id: 'note-a',
    title: 'The vegetable garden',
    score: 0.92,
    label: 'very close',
  ),
  RelatedRow(
    id: 'note-b',
    title: 'Planning spring planting',
    score: 0.83,
    label: 'close',
  ),
];

Widget _page() => Builder(
  builder: (context) => ColoredBox(
    color: context.colors.list,
    child: const MeaningSearchResults(rows: _rows),
  ),
);

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('meaning search results, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _page(),
        name: 'phone_meaning_search_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
      expect(find.text('Found by meaning'), findsOneWidget);
      expect(find.text('The vegetable garden'), findsOneWidget);
      expect(find.text('very close'), findsOneWidget);
    });
  }
}
