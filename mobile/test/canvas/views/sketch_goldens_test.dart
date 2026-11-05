// Kanso — sketch_goldens_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/canvas/views/widgets/sketch_block.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fake_canvas.dart';
import '../../helpers/golden.dart';

const _svg = PreviewSvg(
  markup:
      '<svg xmlns="http://www.w3.org/2000/svg" width="320" height="240"> '
      '<rect width="320" height="240" fill="none"/></svg>',
  width: 320,
  height: 240,
);

CanvasStroke _wave(String id, double y, String color) {
  const steps = 24;
  final top = <double>[];
  final bottom = <double>[];
  for (var i = 0; i <= steps; i++) {
    final px = 20 + i * 11.0;
    final py = y + math.sin(i / 3) * 22;
    top.addAll(<double>[px, py - 3]);
    bottom.insertAll(0, <double>[px, py + 3]);
  }
  return CanvasStroke(
    id: id,
    nodeId: 'ink-1',
    tool: 'pen',
    color: color,
    opacity: 1,
    width: 6,
    bounds: CanvasRect(x: 20, y: y - 28, width: steps * 11, height: 56),
    outline: Float64List.fromList(<double>[...top, ...bottom]),
    known: false,
  );
}

void main() {
  for (final (name, brightness, readOnly) in <(String, Brightness, bool)>[
    ('light', Brightness.light, false),
    ('dark', Brightness.dark, false),
    ('shared', Brightness.light, true),
  ]) {
    testWidgets('a sketch in a note, $name', (tester) async {
      final document = FakeCanvasDocument()
        ..sketch = SketchInfo(
          id: 'sk-1',
          width: 320,
          height: 240,
          readOnly: readOnly,
        );
      document.strokes.addAll(<CanvasStroke>[
        _wave('s1', 80, '#1e1e1e'),
        _wave('s2', 160, '#e03131'),
      ]);
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
        const Scaffold(
          body: SafeArea(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: SketchBlock(
                noteId: 'note-1',
                id: 'sk-1',
                width: 320,
                height: 240,
                svg: _svg,
              ),
            ),
          ),
        ),
        name: 'phone_sketch_$name',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
