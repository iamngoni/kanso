// Kanso — capture_sheet_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_first_row.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/capture/views/widgets/capture_sheet.dart';
import 'package:kanso/core/configs/kanso_icons.dart';

import '../../../helpers/pump.dart';

/// A page with a button that opens the sheet, and what was chosen afterwards.
class _Host extends StatelessWidget {
  const _Host({
    required this.first,
    required this.onSource,
    required this.onFirst,
  });

  final CaptureFirstRow first;
  final ValueChanged<MediaSource> onSource;
  final VoidCallback onFirst;

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Center(
      child: TextButton(
        onPressed: () =>
            CaptureSheet.show(context, first: first, onSource: onSource),
        child: const Text('Open'),
      ),
    ),
  );
}

void main() {
  late CaptureFirstRow first;
  late List<MediaSource> chosen;
  late int firsts;

  setUp(() {
    chosen = <MediaSource>[];
    firsts = 0;
    first = CaptureFirstRow(
      icon: KansoIcons.plus,
      label: 'New note',
      onTap: () => firsts++,
    );
  });

  Future<void> open(WidgetTester tester) async {
    await pumpKanso(
      tester,
      _Host(first: first, onSource: chosen.add, onFirst: () => firsts++),
    );
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
  }

  testWidgets('the sheet offers a new note, the camera, photos and a PDF', (
    tester,
  ) async {
    await open(tester);

    expect(find.text('New note'), findsOneWidget);
    expect(find.text('Take photo'), findsOneWidget);
    expect(find.text('Choose photos'), findsOneWidget);
    expect(find.text('Attach PDF'), findsOneWidget);
  });

  testWidgets('its first row names the first thing, and what it does', (
    tester,
  ) async {
    await open(tester);
    await tester.tap(find.text('New note'));
    await tester.pumpAndSettle();

    expect(firsts, 1);
    expect(chosen, isEmpty);
    expect(find.text('Take photo'), findsNothing);
  });

  for (final (label, source) in <(String, MediaSource)>[
    ('Take photo', MediaSource.camera),
    ('Choose photos', MediaSource.photos),
    ('Attach PDF', MediaSource.pdf),
  ]) {
    testWidgets('"$label" closes the sheet and reports ${source.name}', (
      tester,
    ) async {
      await open(tester);
      await tester.tap(find.text(label));
      await tester.pumpAndSettle();

      expect(chosen, <MediaSource>[source]);
      expect(firsts, 0);
      expect(find.text(label), findsNothing);
    });
  }

  testWidgets('a chosen source is reported once, after the sheet has gone', (
    tester,
  ) async {
    await open(tester);
    await tester.tap(find.text('Take photo'));
    await tester.pumpAndSettle();
    await tester.pumpAndSettle();

    expect(chosen, <MediaSource>[MediaSource.camera]);
  });
}
