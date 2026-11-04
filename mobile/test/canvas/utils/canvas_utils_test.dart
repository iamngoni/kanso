// Kanso — canvas_utils_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/data/canvas_tool.dart';
import 'package:kanso/canvas/utils/canvas_viewport.dart';
import 'package:kanso/canvas/utils/ink_colors.dart';
import 'package:kanso/canvas/utils/pointer_role.dart';
import 'package:kanso/canvas/utils/stroke_path.dart';

const _size = Size(400, 800);

void main() {
  group('CanvasViewport', () {
    test('screen and canvas points map back and forth (pinned seed)', () {
      final random = math.Random(45);
      for (var i = 0; i < 200; i++) {
        final view = CanvasViewport(
          center: Offset(
            random.nextDouble() * 4000 - 2000,
            random.nextDouble() * 4000 - 2000,
          ),
          zoom: 0.05 + random.nextDouble() * 7,
        );
        final p = Offset(random.nextDouble() * 400, random.nextDouble() * 800);
        final back = view.toScreen(view.toCanvas(p, _size), _size);
        expect((back - p).distance, lessThan(1e-6));
      }
    });

    test('a zoom keeps the point under the fingers where it was', () {
      final random = math.Random(45);
      for (var i = 0; i < 200; i++) {
        final view = CanvasViewport(
          center: Offset(random.nextDouble() * 900, random.nextDouble() * 900),
          zoom: 0.2 + random.nextDouble() * 3,
        );
        final at = Offset(random.nextDouble() * 400, random.nextDouble() * 800);
        final factor = 0.3 + random.nextDouble() * 3;
        final next = view.zoomedAt(factor, at, _size);
        final before = view.toCanvas(at, _size);
        final after = next.toCanvas(at, _size);
        if (next.zoom != view.zoom * factor) {
          // Held at a limit: the point may drift but the zoom is in range.
          expect(next.zoom, inInclusiveRange(0.02, 8));
        } else {
          expect((after - before).distance, lessThan(1e-6));
        }
      }
    });

    test('the zoom never leaves its limits', () {
      var view = const CanvasViewport();
      for (var i = 0; i < 40; i++) {
        view = view.zoomedAt(3, Offset.zero, _size);
      }
      expect(view.zoom, CanvasViewport.maxZoom);
      for (var i = 0; i < 80; i++) {
        view = view.zoomedAt(0.3, Offset.zero, _size);
      }
      expect(view.zoom, CanvasViewport.minZoom);
    });

    test('a drag moves the content with the finger', () {
      final view = const CanvasViewport(zoom: 2).panned(const Offset(20, 0));
      expect(view.center, const Offset(-10, 0));
    });

    test('fit shows the bounds whole and an empty canvas its origin', () {
      final fit = CanvasViewport.fit(
        const Rect.fromLTWH(0, 0, 2000, 1000),
        _size,
      );
      final seen = fit.visible(_size);
      expect(seen.contains(Offset.zero), isTrue);
      expect(seen.contains(const Offset(2000, 1000)), isTrue);
      expect(CanvasViewport.fit(null, _size), const CanvasViewport());
    });

    test('fit does not zoom closer than life size', () {
      final fit = CanvasViewport.fit(const Rect.fromLTWH(0, 0, 10, 10), _size);
      expect(fit.zoom, 1);
    });
  });

  group('roleFor', () {
    PointerRole role(
      PointerDeviceKind kind,
      CanvasTool tool, {
      bool stylusSeen = false,
      bool fingerDraws = true,
    }) => roleFor(
      kind: kind,
      tool: tool,
      stylusSeen: stylusSeen,
      fingerDraws: fingerDraws,
    );

    test('a pen draws and its flipped end erases part of a stroke', () {
      expect(
        role(PointerDeviceKind.stylus, CanvasTool.pen),
        PointerRole.ink,
      );
      for (final tool in CanvasTool.values) {
        expect(
          role(PointerDeviceKind.invertedStylus, tool),
          PointerRole.partialErase,
        );
      }
    });

    test('a finger draws until a pen is seen, then only pans', () {
      expect(role(PointerDeviceKind.touch, CanvasTool.pen), PointerRole.ink);
      expect(
        role(PointerDeviceKind.touch, CanvasTool.pen, stylusSeen: true),
        PointerRole.pan,
      );
      expect(
        role(PointerDeviceKind.touch, CanvasTool.pen, fingerDraws: false),
        PointerRole.pan,
      );
    });

    test('a mouse follows the tool, the hand tool pans with anything', () {
      expect(
        role(PointerDeviceKind.mouse, CanvasTool.eraser),
        PointerRole.erase,
      );
      expect(
        role(PointerDeviceKind.mouse, CanvasTool.lasso),
        PointerRole.lasso,
      );
      expect(
        role(PointerDeviceKind.mouse, CanvasTool.text),
        PointerRole.place,
      );
      expect(
        role(PointerDeviceKind.stylus, CanvasTool.hand),
        PointerRole.pan,
      );
    });
  });

  group('penSample', () {
    test('a pen spreads its pressure, a finger draws at one half', () {
      final pen = penSample(
        canvasPoint: const Offset(1, 2),
        kind: PointerDeviceKind.stylus,
        pressure: 0.75,
        pressureMin: 0.5,
        pressureMax: 1,
        tilt: 0,
        orientation: 0,
      );
      expect(pen.pressure, 0.5);
      final finger = penSample(
        canvasPoint: const Offset(1, 2),
        kind: PointerDeviceKind.touch,
        pressure: 1,
        pressureMin: 0,
        pressureMax: 1,
        tilt: 0,
        orientation: 0,
      );
      expect(finger.pressure, 0.5);
      expect(finger.altitude, isNull);
    });

    test('a tilted pen gives azimuth and altitude', () {
      final pen = penSample(
        canvasPoint: Offset.zero,
        kind: PointerDeviceKind.stylus,
        pressure: 1,
        pressureMin: 0,
        pressureMax: 1,
        tilt: math.pi / 4,
        orientation: 1,
      );
      expect(pen.azimuth, 1);
      expect(pen.altitude, closeTo(math.pi / 4, 1e-9));
    });
  });

  group('ink colours', () {
    test('hex colours parse, other things do not', () {
      expect(parseHex('#ff0000'), const Color(0xffff0000));
      expect(parseHex('#f00'), const Color(0xffff0000));
      expect(parseHex('red'), isNull);
      expect(parseHex('#12'), isNull);
      expect(parseHex(null), isNull);
    });

    test('a preset name is a card colour', () {
      expect(cardColor('1'), parseHex('#fb464c'));
      expect(cardColor('#00ff00'), const Color(0xff00ff00));
      expect(cardColor(null), isNull);
    });

    test('black ink turns to the text colour on a dark page only', () {
      const text = Color(0xfff0f0f0);
      expect(inkColor('#1e1e1e', dark: true, text: text), text);
      expect(
        inkColor('#1e1e1e', dark: false, text: text),
        const Color(0xff1e1e1e),
      );
      expect(
        inkColor('#e03131', dark: true, text: text),
        const Color(0xffe03131),
      );
    });
  });

  group('outlinePath', () {
    test('fewer than three points make nothing', () {
      expect(outlinePath(<double>[0, 0, 1, 1]).getBounds(), Rect.zero);
    });

    test('a square fills about its own box', () {
      final bounds = outlinePath(<double>[
        0,
        0,
        20,
        0,
        20,
        20,
        0,
        20,
      ]).getBounds();
      expect(bounds.width, inInclusiveRange(10, 20));
      expect(bounds.height, inInclusiveRange(10, 20));
    });

    test('a shift moves the path', () {
      const outline = <double>[0, 0, 20, 0, 20, 20, 0, 20];
      final a = outlinePath(outline).getBounds();
      final b = outlinePath(outline, shift: const Offset(100, 50)).getBounds();
      expect(b.left - a.left, closeTo(100, 1e-9));
      expect(b.top - a.top, closeTo(50, 1e-9));
    });
  });
}
