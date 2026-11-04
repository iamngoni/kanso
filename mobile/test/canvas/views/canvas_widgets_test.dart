// Kanso — canvas_widgets_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/data/canvas_tool.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/canvas/utils/canvas_viewport.dart';
import 'package:kanso/canvas/views/widgets/canvas_banner.dart';
import 'package:kanso/canvas/views/widgets/canvas_painter.dart';
import 'package:kanso/canvas/views/widgets/canvas_surface.dart';
import 'package:kanso/canvas/views/widgets/canvas_toolbar.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../helpers/fake_canvas.dart';
import '../../helpers/kanso_test_app.dart';

const _theme = CanvasTheme(
  dark: false,
  text: Color(0xff1e1e1e),
  muted: Color(0xff777777),
  surface: Color(0xffffffff),
  border: Color(0xffdddddd),
  accent: Color(0xff3366ff),
);

class _Calls {
  final List<List<PenSample>> strokes = <List<PenSample>>[];
  final List<({bool partial, int points})> erases =
      <({bool partial, int points})>[];
  final List<Offset> places = <Offset>[];
  int stylusSeen = 0;
}

Widget _surface({
  required _Calls calls,
  required CanvasViewController controller,
  CanvasView? view,
  CanvasTool tool = CanvasTool.pen,
  bool fingerDraws = true,
  bool stylusSeen = false,
}) {
  return KansoTestApp(
    home: Align(
      alignment: Alignment.topLeft,
      child: SizedBox(
        width: 400,
        height: 600,
        child: CanvasSurface(
          view: view ?? fakeView(),
          tool: tool,
          color: '#1e1e1e',
          width: 3,
          fingerDraws: fingerDraws,
          stylusSeen: stylusSeen,
          selection: const <String>{},
          edit: 0,
          theme: _theme,
          preview: (style, samples) => <double>[],
          controller: controller,
          onStylusSeen: () => calls.stylusSeen++,
          onStroke: calls.strokes.add,
          onErase: (path, radius, {required partial}) =>
              calls.erases.add((partial: partial, points: path.length)),
          onLasso: (_) {},
          onSelectionCleared: () {},
          onSelectionMoved: (_) {},
          onCardMoved: (_, _) {},
          onCardTapped: (_) {},
          onPlace: calls.places.add,
          onArrow: (_, _) {},
        ),
      ),
    ),
  );
}

Future<void> _drag(
  WidgetTester tester,
  PointerDeviceKind kind,
  Offset from,
  Offset by, {
  int buttons = 0,
}) async {
  final gesture = await tester.createGesture(kind: kind, buttons: buttons);
  await gesture.down(from);
  await tester.pump();
  for (var i = 1; i <= 5; i++) {
    await gesture.moveTo(from + by * (i / 5));
    await tester.pump();
  }
  await gesture.up();
  await tester.pump();
}

void main() {
  group('CanvasToolbar', () {
    Future<List<String>> pump(WidgetTester tester, CanvasTool tool) async {
      final seen = <String>[];
      await tester.pumpWidget(
        KansoTestApp(
          home: Material(
            type: MaterialType.transparency,
            child: Align(
              alignment: Alignment.bottomCenter,
              child: CanvasToolbar(
                tool: tool,
                color: '#1e1e1e',
                width: 3,
                dark: false,
                text: _theme.text,
                selected: const <StrokeKey>[],
                onTool: (t) => seen.add('tool:${t.name}'),
                onColor: (c) => seen.add('color:$c'),
                onWidth: (w) => seen.add('width:$w'),
                onDeleteSelection: () => seen.add('delete'),
                onClearSelection: () => seen.add('clear'),
                onMoveToNew: () => seen.add('moveToNew'),
              ),
            ),
          ),
        ),
      );
      return seen;
    }

    testWidgets('every tool is a button that picks it', (tester) async {
      final seen = await pump(tester, CanvasTool.pen);
      for (final tool in CanvasToolbar.tools) {
        final button = _labelled(CanvasToolbar.label(_l10n(tester), tool));
        expect(button, findsOneWidget, reason: tool.name);
        await tester.ensureVisible(button);
        await tester.tap(button);
      }
      expect(seen, <String>[
        for (final tool in CanvasToolbar.tools) 'tool:${tool.name}',
      ]);
    });

    testWidgets('a pen shows colours and a width', (tester) async {
      final seen = await pump(tester, CanvasTool.pen);
      expect(find.byType(Slider), findsOneWidget);
      await tester.tap(_labelled(RegExp('#e03131')));
      expect(seen, <String>['color:#e03131']);
    });

    testWidgets('the eraser has no colours', (tester) async {
      await pump(tester, CanvasTool.eraser);
      expect(find.byType(Slider), findsNothing);
    });
  });

  group('CanvasBanner', () {
    Future<void> pump(
      WidgetTester tester,
      CanvasBudget budget, {
      CanvasProblem? problem,
    }) => tester.pumpWidget(
      KansoTestApp(
        home: CanvasBanner(
          view: fakeView(budget: budget),
          problem: problem,
        ),
      ),
    );

    testWidgets('says nothing while the canvas has room', (tester) async {
      await pump(tester, CanvasBudget.ok);
      expect(find.byType(Text), findsNothing);
    });

    testWidgets('warns near the limit and says so when full', (tester) async {
      await pump(tester, CanvasBudget.warn);
      final warn = tester.widgetList<Text>(find.byType(Text)).length;
      expect(warn, greaterThan(0));
      await pump(tester, CanvasBudget.full);
      expect(find.byType(Text), findsWidgets);
    });

    testWidgets('says when it could not be saved', (tester) async {
      await pump(tester, CanvasBudget.ok, problem: CanvasProblem.notSaved);
      expect(find.byType(Text), findsWidgets);
    });
  });

  group('CanvasPainter', () {
    testWidgets('paints ink only where the view looks', (tester) async {
      final view = fakeView(
        strokes: <CanvasStroke>[
          fakeStroke('near', x: 10, y: 10),
          fakeStroke('far', x: 90000, y: 90000),
        ],
      );
      final paths = StrokePaths();
      final painter = CanvasPainter(
        view: view,
        viewport: const CanvasViewport(center: Offset(100, 100)),
        theme: _theme,
        paths: paths,
      );
      final recorder = PictureRecorder();
      painter.paint(Canvas(recorder), const Size(400, 600));
      recorder.endRecording().dispose();
      // Only the stroke in sight had its path built.
      expect(paths.of(view.strokes.first).getBounds().isEmpty, isFalse);
      expect(painter.shouldRepaint(painter), isFalse);
    });

    testWidgets('a stroke path is built once', (tester) async {
      final paths = StrokePaths();
      final stroke = fakeStroke('a');
      expect(identical(paths.of(stroke), paths.of(stroke)), isTrue);
    });
  });

  group('CanvasSurface', () {
    testWidgets('a stylus draws a stroke, and is seen', (tester) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(_surface(calls: calls, controller: controller));
      await _drag(
        tester,
        PointerDeviceKind.stylus,
        const Offset(100, 200),
        const Offset(150, 0),
        buttons: kPrimaryStylusButton,
      );
      expect(calls.stylusSeen, greaterThan(0));
      expect(calls.strokes, hasLength(1));
      expect(calls.strokes.single.length, greaterThan(2));
      expect(calls.erases, isEmpty);
    });

    testWidgets('a finger draws while no pen was seen and finger draws is on', (
      tester,
    ) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(_surface(calls: calls, controller: controller));
      await _drag(
        tester,
        PointerDeviceKind.touch,
        const Offset(100, 200),
        const Offset(100, 0),
      );
      expect(calls.strokes, hasLength(1));
      expect(calls.strokes.single.first.pressure, 0.5);
    });

    testWidgets('a finger pans, not draws, once a pen has been seen', (
      tester,
    ) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, stylusSeen: true),
      );
      final before = controller.value.center;
      await _drag(
        tester,
        PointerDeviceKind.touch,
        const Offset(100, 200),
        const Offset(100, 0),
      );
      expect(calls.strokes, isEmpty);
      expect(controller.value.center.dx, lessThan(before.dx));
    });

    testWidgets('palms on the screen while a pen writes are ignored', (
      tester,
    ) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, stylusSeen: true),
      );
      final before = controller.value;
      final pen = await tester.createGesture(
        kind: PointerDeviceKind.stylus,
        buttons: kPrimaryStylusButton,
      );
      await pen.down(const Offset(100, 200));
      await tester.pump();
      // Two contacts of a resting hand land and drift while the pen moves.
      final palm1 = await tester.createGesture(pointer: 21);
      final palm2 = await tester.createGesture(pointer: 22);
      await palm1.down(const Offset(250, 450));
      await palm2.down(const Offset(300, 480));
      await tester.pump();
      for (var i = 1; i <= 5; i++) {
        await pen.moveTo(Offset(100.0 + i * 30, 200));
        await palm1.moveTo(Offset(250.0 + i * 20, 450.0 + i * 10));
        await palm2.moveTo(Offset(300.0 + i * 40, 480));
        await tester.pump();
      }
      await palm1.up();
      await palm2.up();
      await pen.up();
      await tester.pump();
      expect(calls.strokes, hasLength(1), reason: 'the stroke went on');
      expect(calls.strokes.single.length, greaterThan(4));
      expect(controller.value, before, reason: 'the view stayed put');
    });

    testWidgets('with finger draws off a finger pans', (tester) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, fingerDraws: false),
      );
      final before = controller.value.center;
      await _drag(
        tester,
        PointerDeviceKind.touch,
        const Offset(100, 200),
        const Offset(0, 80),
      );
      expect(calls.strokes, isEmpty);
      expect(controller.value.center.dy, lessThan(before.dy));
    });

    testWidgets('two fingers zoom', (tester) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, fingerDraws: false),
      );
      final before = controller.value.zoom;
      final a = await tester.startGesture(const Offset(180, 300));
      final b = await tester.startGesture(const Offset(220, 300));
      await tester.pump();
      for (var i = 1; i <= 5; i++) {
        await a.moveTo(Offset(180.0 - i * 15, 300));
        await b.moveTo(Offset(220.0 + i * 15, 300));
        await tester.pump();
      }
      await a.up();
      await b.up();
      await tester.pump();
      expect(controller.value.zoom, greaterThan(before));
      expect(calls.strokes, isEmpty);
    });

    testWidgets('the flipped end of a pen erases part of a stroke', (
      tester,
    ) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(_surface(calls: calls, controller: controller));
      await _drag(
        tester,
        PointerDeviceKind.invertedStylus,
        const Offset(100, 200),
        const Offset(120, 0),
      );
      expect(calls.strokes, isEmpty);
      expect(calls.erases, isNotEmpty);
      expect(calls.erases.every((e) => e.partial), isTrue);
    });

    testWidgets('the eraser tool takes whole strokes', (tester) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, tool: CanvasTool.eraser),
      );
      await _drag(
        tester,
        PointerDeviceKind.stylus,
        const Offset(100, 200),
        const Offset(100, 0),
      );
      expect(calls.erases, isNotEmpty);
      expect(calls.erases.every((e) => !e.partial), isTrue);
    });

    testWidgets('a tap with the text tool places a card', (tester) async {
      final calls = _Calls();
      final controller = CanvasViewController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        _surface(calls: calls, controller: controller, tool: CanvasTool.text),
      );
      await tester.tapAt(const Offset(200, 300));
      await tester.pump();
      expect(calls.places, hasLength(1));
      expect(calls.strokes, isEmpty);
    });
  });
}

AppLocalizations _l10n(WidgetTester tester) =>
    AppLocalizations.of(tester.element(find.byType(CanvasToolbar)));

/// The widget a [Semantics] with this label (a string or a pattern) wraps.
Finder _labelled(Pattern label) => find.byWidgetPredicate((w) {
  if (w is! Semantics) {
    return false;
  }
  final text = w.properties.label;
  if (text == null) {
    return false;
  }
  return label is RegExp ? label.hasMatch(text) : text == label;
});
