// Kanso — canvas_goldens_test.dart
// Created 2026-11-04.
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

/// A pen stroke along a wave, as an outline polygon `thick` wide.
CanvasStroke _wave(
  String id, {
  required double x,
  required double y,
  required String color,
  double thick = 6,
  String tool = 'pen',
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
    tool: tool,
    color: color,
    opacity: tool == 'highlighter' ? 0.4 : 1,
    width: thick,
    bounds: CanvasRect(x: x, y: y - 30, width: steps * 8, height: 60),
    outline: Float64List.fromList(<double>[...top, ...bottom]),
    known: false,
  );
}

CanvasCard _card(String id, double x, double y, String text, {String? color}) =>
    CanvasCard(
      id: id,
      kind: 'text',
      rect: CanvasRect(x: x, y: y, width: 180, height: 90),
      text: text,
      color: color,
      missing: false,
    );

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('the canvas screen, ${brightness.name}', (tester) async {
      final document = FakeCanvasDocument(
        cards: <CanvasCard>[
          _card('c1', 20, 20, 'Ideas for the trip', color: '4'),
          _card('c2', 240, 220, 'Book the train'),
        ],
      );
      document.strokes.addAll(<CanvasStroke>[
        _wave('s1', x: 20, y: 200, color: '#1e1e1e'),
        _wave('s2', x: 40, y: 320, color: '#e03131', thick: 8),
        _wave('s3', x: 30, y: 420, color: '#f08c00', tool: 'highlighter'),
      ]);
      document
        ..dirty = false
        ..canUndo = true;
      final repository = FakeCanvasRepository(document: document);
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
        name: 'phone_canvas_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
