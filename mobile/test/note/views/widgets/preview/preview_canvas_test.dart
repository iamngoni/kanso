// Kanso — preview_canvas_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note/views/widgets/preview/native_preview.dart';

import '../../../../helpers/golden.dart';
import '../../../../helpers/kanso_test_app.dart';

const PaletteDto _light = PaletteDto(
  dark: false,
  background: 0xFBFAF7,
  text: 0x2B2A27,
  muted: 0x6F6B64,
  accent: 0x8A6A3D,
  border: 0xE4E0D8,
  surface: 0xF3F1EC,
  quote: 0x6F6B64,
  taskDone: 0x6F6B64,
  inlineCodeBackground: 0xF3F1EC,
);

const PaletteDto _dark = PaletteDto(
  dark: true,
  background: 0x1E1E1C,
  text: 0xE6E3DD,
  muted: 0x9C978E,
  accent: 0xD2AE78,
  border: 0x3A3935,
  surface: 0x282826,
  quote: 0x9C978E,
  taskDone: 0x9C978E,
  inlineCodeBackground: 0x282826,
);

const String _svg =
    '<svg xmlns="http://www.w3.org/2000/svg" width="240" height="120" '
    'viewBox="0 0 240 120"><rect x="1" y="1" width="238" height="118" rx="8" '
    'fill="#f3f1ec" stroke="#e4e0d8" stroke-width="1.5"/> '
    '<text x="16" y="40" font-size="16" fill="#2b2a27">Launch plan</text> '
    '</svg>';

PreviewBlock _canvas(PreviewCanvasShown shown) =>
    PreviewBlock.canvas(note: 'note:AbCd1234', shown: shown);

const PreviewCanvasShown _drawn = PreviewCanvasShown.drawing(
  svg: PreviewSvg(markup: _svg, width: 240, height: 120),
  notices: <PreviewCanvasNotice>[],
);

const PreviewCanvasShown _huge = PreviewCanvasShown.drawing(
  svg: PreviewSvg(markup: _svg, width: 240, height: 120),
  notices: <PreviewCanvasNotice>[PreviewCanvasNotice.wholeCanvas],
);

Widget _page(List<PreviewBlock> blocks, PaletteDto palette, List<String> taps) {
  return Scaffold(
    body: SafeArea(
      child: CustomScrollView(
        slivers: <Widget>[
          SliverPadding(
            padding: const EdgeInsets.all(16),
            sliver: NativePreview(
              blocks: blocks,
              palette: palette,
              onLink: taps.add,
              onEdit: () {},
            ),
          ),
        ],
      ),
    ),
  );
}

void main() {
  late List<String> taps;

  setUp(() => taps = <String>[]);

  Future<void> pump(WidgetTester tester, List<PreviewBlock> blocks) async {
    await tester.pumpWidget(
      KansoTestApp(home: _page(blocks, _light, taps)),
    );
    await tester.pump();
  }

  group('a canvas in a note', () {
    testWidgets('is its picture, and a tap opens the canvas', (tester) async {
      await pump(tester, <PreviewBlock>[_canvas(_drawn)]);
      expect(find.byType(SvgPicture), findsOneWidget);
      await tester.tap(find.byType(SvgPicture));
      // The bare ID, as every note link carries it: `note:` again would
      // name no note.
      expect(taps, <String>['inkdrop://note/AbCd1234']);
    });

    testWidgets('a large whole-canvas embed says so and still draws', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[_canvas(_huge)]);
      expect(find.byType(SvgPicture), findsOneWidget);
      expect(
        find.text('This embeds the whole canvas. Name a group to embed less.'),
        findsOneWidget,
      );
    });

    testWidgets('a locked canvas is a placeholder with no picture', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        _canvas(const PreviewCanvasShown.locked()),
      ]);
      expect(find.text('Locked canvas'), findsOneWidget);
      // It opens like any locked note, which asks for the lock.
      await tester.tap(find.text('Locked canvas'));
      expect(taps, <String>['inkdrop://note/AbCd1234']);
      // The lock glyph is an icon; no drawing of the canvas is.
      expect(
        find.byWidgetPredicate((w) => w is SvgPicture && w.width == 240),
        findsNothing,
      );
    });

    testWidgets('a missing, foreign or broken canvas says why', (tester) async {
      await pump(tester, <PreviewBlock>[
        _canvas(const PreviewCanvasShown.missing()),
        _canvas(const PreviewCanvasShown.notCanvas()),
        _canvas(const PreviewCanvasShown.invalid(message: 'bad')),
      ]);
      expect(find.text('Canvas not found'), findsOneWidget);
      expect(find.text("This note isn't a canvas"), findsOneWidget);
      expect(find.text("Couldn't draw this canvas"), findsOneWidget);
      // None of them opens anything.
      await tester.tap(find.text('Canvas not found'));
      expect(taps, isEmpty);
    });
  });

  for (final brightness in Brightness.values) {
    final name = brightness.name;
    final palette = brightness == Brightness.dark ? _dark : _light;
    final shapes = <String, PreviewCanvasShown>{
      'drawing': _drawn,
      'locked': const PreviewCanvasShown.locked(),
      'huge': _huge,
    };
    for (final entry in shapes.entries) {
      testWidgets('${entry.key} embed, $name', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          _page(<PreviewBlock>[_canvas(entry.value)], palette, taps),
          name: 'phone_canvas_embed_${entry.key}_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });
    }
  }
}
