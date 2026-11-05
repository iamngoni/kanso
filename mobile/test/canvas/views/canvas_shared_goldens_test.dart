// Kanso — canvas_shared_goldens_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/canvas/views/pages/canvas_page.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fake_canvas.dart';
import '../../helpers/golden.dart';

CanvasStroke _wave(
  String id, {
  required double x,
  required double y,
  required String color,
  double thick = 6,
}) {
  const steps = 24;
  final top = <double>[];
  final bottom = <double>[];
  for (var i = 0; i <= steps; i++) {
    final px = x + i * 8;
    final py = y + math.sin(i / 3) * 24;
    top.addAll(<double>[px, py - thick / 2]);
    bottom.insertAll(0, <double>[px, py + thick / 2]);
  }
  return CanvasStroke(
    id: id,
    nodeId: 'ink-1',
    tool: 'pen',
    color: color,
    opacity: 1,
    width: thick,
    bounds: CanvasRect(x: x, y: y - 30, width: steps * 8, height: 60),
    outline: Float64List.fromList(<double>[...top, ...bottom]),
    known: false,
  );
}

CanvasCard _card(String id, double x, double y, String text) => CanvasCard(
  id: id,
  kind: 'text',
  rect: CanvasRect(x: x, y: y, width: 180, height: 90),
  text: text,
  missing: false,
);

/// People on the canvas: one low and to the left, one high, one at the
/// right edge, whose name opens to the left of its dot.
const _people = <CanvasPointerRow>[
  CanvasPointerRow(
    accountId: 'acct-ada',
    name: 'Ada',
    x: 70,
    y: 130,
    tool: 'pen',
    colour: 0,
    colourLight: '#2F6AAE',
    colourDark: '#7FB0EC',
  ),
  CanvasPointerRow(
    accountId: 'acct-grace',
    name: 'Grace Hopper',
    x: 385,
    y: 60,
    tool: 'lasso',
    colour: 3,
    colourLight: '#7550AE',
    colourDark: '#B89CEE',
  ),
  CanvasPointerRow(
    accountId: 'acct-linus',
    name: 'Linus',
    x: 250,
    y: 250,
    tool: 'eraser',
    colour: 2,
    colourLight: '#A6531A',
    colourDark: '#EDA160',
  ),
];

FakeCanvasDocument _document({required bool readOnly}) {
  final document =
      FakeCanvasDocument(
          cards: <CanvasCard>[
            _card('c1', 20, 20, 'Ideas for the trip'),
            _card('c2', 200, 190, 'Book the train'),
          ],
        )
        ..shared = true
        ..readOnly = readOnly
        ..others = _people
        ..dirty = false
        ..canUndo = !readOnly;
  document.strokes.addAll(<CanvasStroke>[
    _wave('s1', x: 20, y: 150, color: '#1e1e1e'),
    _wave('s2', x: 40, y: 270, color: '#e03131', thick: 8),
  ]);
  return document;
}

void main() {
  for (final brightness in Brightness.values) {
    for (final readOnly in <bool>[false, true]) {
      final kind = readOnly ? 'view_only' : 'pointers';
      testWidgets('a shared canvas, $kind, ${brightness.name}', (tester) async {
        final repository = FakeCanvasRepository(
          document: _document(readOnly: readOnly),
        );
        getIt
          ..allowReassignment = true
          ..registerSingleton<CanvasRepository>(repository)
          ..registerFactory<CanvasBloc>(() => CanvasBloc(repository));
        addTearDown(() {
          getIt
            ..unregister<CanvasRepository>()
            ..unregister<CanvasBloc>();
        });
        await expectGoldenOnBothPlatforms(
          tester,
          const CanvasPage(noteId: 'canvas-1'),
          name: 'phone_canvas_shared_${kind}_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
          // The room's first tick brings the people in.
          interact: (tester) async {
            await tester.pump(canvasPulseEvery * 2);
            await tester.pump(const Duration(milliseconds: 400));
          },
        );
      });
    }
  }
}
