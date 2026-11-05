// Kanso — sketch_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/canvas/views/widgets/sketch_block.dart';
import 'package:kanso/canvas/views/widgets/sketch_size_sheet.dart';
import 'package:kanso/canvas/views/widgets/sketch_surface.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fake_canvas.dart';
import '../../helpers/kanso_test_app.dart';

const _svg = PreviewSvg(
  markup:
      '<svg xmlns="http://www.w3.org/2000/svg" width="480" height="320"> '
      '<rect width="480" height="320" fill="#f4f1ec"/></svg>',
  width: 480,
  height: 320,
);

FakeCanvasDocument _document({bool readOnly = false}) =>
    FakeCanvasDocument()
      ..sketch = SketchInfo(
        id: 'sk-1',
        width: 480,
        height: 320,
        readOnly: readOnly,
      );

Future<FakeCanvasRepository> _register(FakeCanvasDocument document) async {
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
  return repository;
}

Widget _block() => const KansoTestApp(
  home: Material(
    type: MaterialType.transparency,
    child: Padding(
      padding: EdgeInsets.all(8),
      child: SketchBlock(
        noteId: 'note-1',
        id: 'sk-1',
        width: 480,
        height: 320,
        svg: _svg,
      ),
    ),
  ),
);

Future<void> _draw(
  WidgetTester tester,
  PointerDeviceKind kind, {
  Offset by = const Offset(60, 40),
}) async {
  final surface = find.byType(SketchSurface);
  final from = tester.getTopLeft(surface) + const Offset(40, 40);
  final gesture = await tester.createGesture(kind: kind);
  await gesture.down(from);
  await tester.pump();
  for (var i = 1; i <= 4; i++) {
    await gesture.moveTo(from + by * (i / 4));
    await tester.pump();
  }
  await gesture.up();
  await tester.pump();
}

void main() {
  testWidgets('opens its sketch in the note and draws on it', (tester) async {
    final document = _document();
    final repository = await _register(document);
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    expect(repository.sketchOpens, <String>['note-1/sk-1']);
    expect(find.byType(SketchSurface), findsOneWidget);
  });

  testWidgets('a pen stroke is written two seconds later, not per point', (
    tester,
  ) async {
    final document = _document();
    await _register(document);
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    await _draw(tester, PointerDeviceKind.stylus);
    expect(document.calls, contains('draw'));
    expect(document.saves, 0);
    await tester.pump(const Duration(seconds: 1));
    expect(document.saves, 0);
    await tester.pump(const Duration(seconds: 2));
    await tester.pump();
    expect(document.saves, 1);
  });

  testWidgets('a finger scrolls the note instead of drawing', (tester) async {
    final document = _document();
    await _register(document);
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    await _draw(tester, PointerDeviceKind.touch);
    expect(document.calls.where((c) => c == 'draw'), isEmpty);
  });

  testWidgets('a full sketch says so', (tester) async {
    final document = _document()
      ..drawError = const ApplicationError.invalid(message: 'full');
    await _register(document);
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    await _draw(tester, PointerDeviceKind.mouse);
    await tester.pumpAndSettle();
    expect(
      find.text('This sketch is full. Open it as a canvas to keep drawing.'),
      findsOneWidget,
    );
  });

  testWidgets('a shared note draws as a picture and copies as a canvas', (
    tester,
  ) async {
    final document = _document(readOnly: true);
    await _register(document);
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    expect(find.byType(SketchSurface), findsNothing);
    expect(
      find.text('Shared note: copy it to a canvas to draw together.'),
      findsOneWidget,
    );
    await tester.tap(find.text('Copy as canvas'));
    await tester.pumpAndSettle();
    expect(document.canvasTitles, <String>['Sketch sketch']);
    expect(document.calls.where((c) => c == 'draw'), isEmpty);
  });

  testWidgets('a locked note shows the picture only', (tester) async {
    final repository = FakeCanvasRepository(
      openError: const ApplicationError.locked(message: 'locked'),
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
    await tester.pumpWidget(_block());
    await tester.pumpAndSettle();
    expect(find.byType(SketchSurface), findsNothing);
  });

  test('the sizes are the three the design offers', () {
    expect(sketchSizes, const <(int, int)>[(320, 240), (480, 320), (640, 480)]);
  });

  // Keeps the either/or in `dartz` imports honest for the fakes.
  test('a fake open that fails is a Left', () async {
    final repository = FakeCanvasRepository(
      openError: const ApplicationError.locked(message: 'x'),
    );
    final opened = await repository.openSketch('n', 's');
    expect(opened.isLeft(), isTrue);
    expect(const Right<int, int>(1).isRight(), isTrue);
  });
}
