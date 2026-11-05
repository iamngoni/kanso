// Kanso — canvas_shared_widgets_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/data/canvas_tool.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/canvas/utils/canvas_viewport.dart';
import 'package:kanso/canvas/views/pages/canvas_page.dart';
import 'package:kanso/canvas/views/widgets/canvas_painter.dart';
import 'package:kanso/canvas/views/widgets/canvas_pointers.dart';
import 'package:kanso/canvas/views/widgets/canvas_share_bar.dart';
import 'package:kanso/canvas/views/widgets/canvas_surface.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/injection.dart';

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

const ada = CanvasPointerRow(
  accountId: 'acct-ada',
  name: 'Ada',
  x: 100,
  y: 80,
  tool: 'pen',
  colour: 0,
  colourLight: '#2F6AAE',
  colourDark: '#7FB0EC',
);

const grace = CanvasPointerRow(
  accountId: 'acct-grace',
  name: 'Grace Hopper',
  x: 370,
  y: 200,
  tool: 'lasso',
  colour: 3,
  colourLight: '#7550AE',
  colourDark: '#B89CEE',
);

/// What the surface reported, in order.
class _Told {
  final List<String> log = <String>[];
  final List<Offset?> pointer = <Offset?>[];
  final List<List<PenSample>> streamed = <List<PenSample>>[];
}

Widget _surface({
  required _Told told,
  required CanvasViewController controller,
  bool shared = true,
  bool readOnly = false,
  int erasedUnderPen = 0,
  List<CanvasPointerRow> pointers = const <CanvasPointerRow>[],
}) {
  return KansoTestApp(
    home: Align(
      alignment: Alignment.topLeft,
      child: SizedBox(
        width: 400,
        height: 600,
        child: CanvasSurface(
          view: fakeView(shared: shared, readOnly: readOnly),
          tool: CanvasTool.pen,
          color: '#1e1e1e',
          width: 3,
          fingerDraws: true,
          stylusSeen: false,
          selection: const <String>{},
          edit: 0,
          theme: _theme,
          preview: (style, samples) => <double>[],
          controller: controller,
          shared: shared,
          readOnly: readOnly,
          erasedUnderPen: erasedUnderPen,
          pointers: pointers,
          onStylusSeen: () {},
          onStroke: (_) => told.log.add('stroke'),
          onErase: (path, radius, {required partial}) => told.log.add('erase'),
          onLasso: (_) {},
          onSelectionCleared: () {},
          onSelectionMoved: (_) {},
          onCardMoved: (_, _) {},
          onCardTapped: (_) => told.log.add('card'),
          onPlace: (_) {},
          onArrow: (_, _) {},
          onGesture: ({required active}) =>
              told.log.add(active ? 'hand down' : 'hand up'),
          onStream: told.streamed.add,
          onStrokeCancelled: () => told.log.add('cancelled'),
          onPointer: told.pointer.add,
        ),
      ),
    ),
  );
}

Future<void> _drag(
  WidgetTester tester,
  Offset from,
  Offset by, {
  PointerDeviceKind kind = PointerDeviceKind.touch,
  int moves = 5,
}) async {
  final gesture = await tester.createGesture(kind: kind);
  await gesture.down(from);
  await tester.pump();
  for (var i = 1; i <= moves; i++) {
    await gesture.moveTo(from + by * (i / moves));
    await tester.pump();
  }
  await gesture.up();
  await tester.pump();
}

AppLocalizations _l10n(WidgetTester tester) =>
    AppLocalizations.of(tester.element(find.byType(Directionality).last));

/// The widget a [Semantics] with this label wraps.
Finder _labelled(String label) => find.byWidgetPredicate(
  (w) => w is Semantics && w.properties.label == label,
);

void main() {
  late CanvasViewController controller;

  setUp(() => controller = CanvasViewController());
  tearDown(() => controller.dispose());

  group('CanvasSurface on a shared canvas', () {
    testWidgets('says when a hand is down and when the last one lifts, after '
        'the stroke it made', (tester) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      await _drag(tester, const Offset(100, 100), const Offset(120, 40));
      expect(told.log, <String>['hand down', 'stroke', 'hand up']);
    });

    testWidgets('a pan or a tap that edits nothing still ends a gesture', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(
        _surface(told: told, controller: controller, readOnly: true),
      );
      await _drag(tester, const Offset(100, 100), const Offset(60, 0));
      expect(told.log, <String>['hand down', 'hand up']);
      told.log.clear();
      final tap = await tester.createGesture();
      await tap.down(const Offset(150, 150));
      await tap.up();
      await tester.pump();
      expect(told.log, <String>['hand down', 'hand up']);
    });

    testWidgets('a two-finger gesture is one hand-down until both lift', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      final one = await tester.createGesture(pointer: 1);
      final two = await tester.createGesture(pointer: 2);
      await one.down(const Offset(100, 100));
      await two.down(const Offset(200, 100));
      await one.moveTo(const Offset(90, 100));
      await one.up();
      await tester.pump();
      expect(told.log, <String>['hand down']);
      await two.up();
      await tester.pump();
      expect(told.log, <String>['hand down', 'hand up']);
    });

    testWidgets('streams the stroke being drawn, a chunk at a time', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      final gesture = await tester.createGesture();
      await gesture.down(const Offset(100, 100));
      await gesture.moveTo(const Offset(110, 100));
      await tester.pump();
      expect(told.streamed, hasLength(1));
      expect(told.streamed.single.length, 2);
      // Moves inside the same 150 ms are not sent again.
      await gesture.moveTo(const Offset(120, 100));
      await tester.pump();
      expect(told.streamed, hasLength(1));
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 200)),
      );
      await gesture.moveTo(const Offset(130, 100));
      await tester.pump();
      expect(told.streamed, hasLength(2));
      expect(told.streamed.last.length, 4, reason: 'all the samples so far');
      await gesture.up();
      await tester.pump();
      expect(told.log, isNot(contains('cancelled')));
    });

    testWidgets('a second finger takes the stroke back from the others', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      final one = await tester.createGesture(pointer: 1);
      await one.down(const Offset(100, 100));
      await one.moveTo(const Offset(120, 100));
      await tester.pump();
      expect(told.streamed, hasLength(1));
      final two = await tester.createGesture(pointer: 2);
      await two.down(const Offset(250, 250));
      await tester.pump();
      expect(told.log, contains('cancelled'));
      await one.up();
      await two.up();
      await tester.pump();
      expect(told.log, isNot(contains('stroke')));
    });

    testWidgets('a stroke someone erased under the pen is dropped quietly', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      final gesture = await tester.createGesture();
      await gesture.down(const Offset(100, 100));
      await gesture.moveTo(const Offset(120, 100));
      await tester.pump();
      await tester.pumpWidget(
        _surface(told: told, controller: controller, erasedUnderPen: 1),
      );
      await gesture.moveTo(const Offset(140, 100));
      await gesture.up();
      await tester.pump();
      // Nothing to take back (the room has the erase), and no stroke made.
      expect(told.log, isNot(contains('cancelled')));
      expect(told.log, isNot(contains('stroke')));
      expect(told.log.last, 'hand up');
    });

    testWidgets('tells the room where the pointer is, and when it leaves', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(_surface(told: told, controller: controller));
      await _drag(tester, const Offset(100, 100), const Offset(50, 0));
      expect(told.pointer.first, isNotNull);
      expect(told.pointer.last, isNull, reason: 'it lifted');
    });

    testWidgets('a canvas that is not shared tells nobody anything', (
      tester,
    ) async {
      final told = _Told();
      await tester.pumpWidget(
        _surface(told: told, controller: controller, shared: false),
      );
      await _drag(tester, const Offset(100, 100), const Offset(120, 40));
      expect(told.streamed, isEmpty);
      expect(told.pointer.whereType<Offset>(), isEmpty);
    });

    testWidgets("a viewer's pen and finger only pan and zoom", (tester) async {
      final told = _Told();
      await tester.pumpWidget(
        _surface(told: told, controller: controller, readOnly: true),
      );
      final before = controller.value;
      await _drag(
        tester,
        const Offset(200, 200),
        const Offset(80, 0),
        kind: PointerDeviceKind.stylus,
      );
      expect(told.log, isNot(contains('stroke')));
      expect(told.streamed, isEmpty);
      expect(controller.value, isNot(before), reason: 'it panned');
    });

    testWidgets("shows the others' pointers with their names", (tester) async {
      final told = _Told();
      await tester.pumpWidget(
        _surface(
          told: told,
          controller: controller,
          pointers: const <CanvasPointerRow>[ada],
        ),
      );
      await tester.pump(const Duration(milliseconds: 300));
      expect(find.text('Ada'), findsOneWidget);
      expect(
        _labelled(_l10n(tester).canvasPointerLabel('Ada')),
        findsOneWidget,
      );
    });
  });

  group('CanvasPointers', () {
    Future<void> pump(
      WidgetTester tester,
      List<CanvasPointerRow> pointers, {
      CanvasViewport viewport = const CanvasViewport(center: Offset(200, 150)),
      bool dark = false,
    }) => tester.pumpWidget(
      KansoTestApp(
        home: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: 400,
            height: 300,
            child: CanvasPointers(
              pointers: pointers,
              viewport: viewport,
              size: const Size(400, 300),
              dark: dark,
              ring: Colors.white,
            ),
          ),
        ),
      ),
    );

    testWidgets('puts a dot where the pointer is, at any zoom', (tester) async {
      await pump(tester, const <CanvasPointerRow>[ada]);
      await tester.pumpAndSettle();
      final near = tester.getTopLeft(find.text('Ada'));
      await pump(
        tester,
        const <CanvasPointerRow>[ada],
        viewport: const CanvasViewport(center: Offset(200, 150), zoom: 2),
      );
      await tester.pumpAndSettle();
      final far = tester.getTopLeft(find.text('Ada'));
      expect(far, isNot(near), reason: 'the mark follows the canvas point');
      // The chip is the same size at any zoom.
      expect(tester.getSize(find.text('Ada')), isNotNull);
    });

    testWidgets('leaves out a pointer that is far off the screen', (
      tester,
    ) async {
      await pump(tester, const <CanvasPointerRow>[
        CanvasPointerRow(
          accountId: 'x',
          name: 'Far',
          x: 9000,
          y: 9000,
          tool: 'pen',
          colour: 1,
          colourLight: '#2D7A4D',
          colourDark: '#76C59C',
        ),
      ]);
      expect(find.text('Far'), findsNothing);
    });

    testWidgets('a name near the right edge opens to the left of its dot', (
      tester,
    ) async {
      await pump(tester, const <CanvasPointerRow>[grace]);
      await tester.pumpAndSettle();
      final name = tester.getRect(find.text('Grace Hopper'));
      expect(name.right, lessThanOrEqualTo(400));
    });

    test('reads a colour, and is grey for anything else', () {
      expect(pointerColor('#2F6AAE'), const Color(0xff2f6aae));
      expect(pointerColor('nope'), const Color(0xff7a808a));
      expect(pointerColor(''), const Color(0xff7a808a));
    });
  });

  group('CanvasShareBar', () {
    Future<void> pump(
      WidgetTester tester,
      CanvasView view,
      List<CanvasPointerRow> pointers,
    ) => tester.pumpWidget(
      KansoTestApp(
        home: Material(
          child: CanvasShareBar(view: view, pointers: pointers),
        ),
      ),
    );

    testWidgets('a private canvas has none', (tester) async {
      await pump(tester, fakeView(), const <CanvasPointerRow>[]);
      expect(find.byType(Text), findsNothing);
    });

    testWidgets('says shared, and how many others are here', (tester) async {
      await pump(
        tester,
        fakeView(shared: true),
        const <CanvasPointerRow>[ada, grace],
      );
      final l10n = _l10n(tester);
      expect(find.text(l10n.canvasSharedLabel), findsOneWidget);
      expect(find.text(l10n.canvasOthersHere(2)), findsOneWidget);
    });

    testWidgets('says view only for a viewer', (tester) async {
      await pump(
        tester,
        fakeView(shared: true, readOnly: true),
        const <CanvasPointerRow>[],
      );
      final l10n = _l10n(tester);
      expect(
        find.text('${l10n.canvasSharedLabel} · ${l10n.canvasViewOnlyLabel}'),
        findsOneWidget,
      );
    });
  });

  group('CanvasPage on a shared canvas', () {
    Future<FakeCanvasDocument> open(
      WidgetTester tester, {
      required bool readOnly,
    }) async {
      final document = FakeCanvasDocument()
        ..shared = true
        ..readOnly = readOnly
        ..strokes.add(fakeStroke('s1', x: 20, y: 20))
        ..others = <CanvasPointerRow>[ada];
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
      await tester.pumpWidget(
        const KansoTestApp(home: CanvasPage(noteId: 'canvas-1')),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 400));
      return document;
    }

    testWidgets('a viewer has no tools, no undo, and is told so', (
      tester,
    ) async {
      await open(tester, readOnly: true);
      final l10n = _l10n(tester);
      expect(find.text(l10n.canvasViewOnlyLabel), findsWidgets);
      expect(find.text(l10n.canvasViewOnlyMessage), findsOneWidget);
      expect(_labelled(l10n.canvasToolPen), findsNothing);
      expect(_labelled(l10n.canvasUndo), findsNothing);
      expect(_labelled(l10n.canvasRedo), findsNothing);
      // They can still look: fit and more are there.
      expect(_labelled(l10n.canvasFit), findsOneWidget);
      expect(find.text('Ada'), findsOneWidget);
    });

    testWidgets('an editor keeps the tools and sees who is here', (
      tester,
    ) async {
      await open(tester, readOnly: false);
      final l10n = _l10n(tester);
      expect(find.text(l10n.canvasViewOnlyMessage), findsNothing);
      expect(_labelled(l10n.canvasToolPen), findsOneWidget);
      expect(find.text(l10n.canvasSharedLabel), findsOneWidget);
      expect(find.text(l10n.canvasOthersHere(1)), findsOneWidget);
      expect(find.text('Ada'), findsOneWidget);
    });
  });
}
