// Kanso — renderer_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/renderer.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// A small theme file: a name, whether it is light or dark, and three colours;
/// the rest is the defaults'.
const String themeJson = '''
{
  "schema_version": 1,
  "name": "Suite Light",
  "appearance": "light",
  "ui": {"editor": "#FCFAF4", "text": "#26241F", "accent": "#5E7895"}
}
''';

/// Drawing, through the bridge: every call of `KansoRenderer`.
void rendererSuite(OpenLibrary open) {
  group('KansoRenderer', () {
    late PaletteDto palette;

    setUpAll(() async {
      palette = (await KansoRenderer.themeTokens(themeJson: themeJson)).palette;
    });

    test('a theme file is fitted for drawing', () async {
      final tokens = await KansoRenderer.themeTokens(themeJson: themeJson);
      expect(tokens.name, 'Suite Light');
      expect(tokens.dark, isFalse);
      expect(tokens.ui.map((color) => color.name), contains('editor'));
      expect(tokens.syntax, isNotEmpty);
      expect(tokens.palette.background, 0xFCFAF4);
      await expectLater(
        KansoRenderer.themeTokens(themeJson: 'not json'),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test("the built-in themes are the desktop's, fitted", () async {
      final themes = await KansoRenderer.builtinThemes();
      expect(themes.length, greaterThan(20));
      expect(themes.take(2).map((theme) => theme.id), <String>[
        'kanso-light',
        'kanso-dark',
      ]);
      expect(themes.map((theme) => theme.id).toSet(), hasLength(themes.length));
      final nord = themes.firstWhere((theme) => theme.id == 'nord');
      expect(nord.dark, isTrue);
      expect(nord.name, isNotEmpty);
      expect(nord.license, isNotEmpty);
      final tokens = await KansoRenderer.builtinThemeTokens(id: 'nord');
      expect(tokens.name, nord.tokens.name);
      expect(tokens.caret, nord.tokens.caret);
      expect(tokens.ui.length, nord.tokens.ui.length);
      expect(tokens.ui.first.rgb, nord.tokens.ui.first.rgb);
      expect(tokens.palette.background, nord.tokens.palette.background);
      await expectLater(
        KansoRenderer.builtinThemeTokens(id: 'nope'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test("the built-in icon themes are the desktop's, Lucide first", () async {
      final themes = await KansoRenderer.iconThemes();
      expect(themes.length, greaterThan(20));
      expect(themes.first.id, 'lucide');
      expect(themes.first.name, 'Lucide');
      expect(themes.map((theme) => theme.id).toSet(), hasLength(themes.length));
      final phosphor = themes.firstWhere((theme) => theme.id == 'phosphor');
      expect(phosphor.name, 'Phosphor');
      expect(phosphor.license, 'MIT');
      expect(phosphor.preview, hasLength(5));
      expect(phosphor.preview.any((icon) => icon.svg != null), isTrue);
      expect(themes.first.preview.every((icon) => icon.svg == null), isTrue);
    });

    test(
      'a theme draws the icons it has and leaves the rest to Lucide',
      () async {
        final lucide = await KansoRenderer.iconThemeIcons(id: 'lucide');
        expect(lucide, isNotEmpty);
        expect(lucide.every((icon) => icon.svg == null), isTrue);
        final phosphor = await KansoRenderer.iconThemeIcons(id: 'phosphor');
        expect(phosphor.map((icon) => icon.name), lucide.map((i) => i.name));
        final drawn = phosphor.where((icon) => icon.svg != null).toList();
        expect(drawn.length, greaterThan(40));
        for (final icon in drawn) {
          expect(icon.svg, contains('<svg'), reason: icon.name);
        }
        await expectLater(
          KansoRenderer.iconThemeIcons(id: 'nope'),
          throwsA(isA<KansoError_NotFound>()),
        );
      },
    );

    test("source is styled in a theme's colours", () async {
      const text = '# Title\n\nSome `code` here.\n';
      final runs = await KansoRenderer.styledRuns(
        text: text,
        start: 0,
        end: text.length,
        themeId: 'nord',
      );
      expect(runs, isNotEmpty);
      for (final run in runs) {
        expect(run.range.start, lessThan(run.range.end));
        expect(run.range.end, lessThanOrEqualTo(text.length));
        expect(
          run.color != null || run.italic != null || run.weight != null,
          isTrue,
        );
      }
      final heading = (await KansoRenderer.builtinThemeTokens(
        id: 'nord',
      )).syntax.firstWhere((style) => style.token == 'heading');
      expect(runs.any((run) => run.color == heading.color), isTrue);
      await expectLater(
        KansoRenderer.styledRuns(
          text: text,
          start: 0,
          end: text.length,
          themeId: 'nope',
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test(
      'highlighting gives runs inside the range, with their tokens',
      () async {
        const text = '# Title\n\nSome `code` here.\n';
        final spans = await KansoRenderer.highlight(
          text: text,
          start: 0,
          end: text.length,
        );
        expect(spans, isNotEmpty);
        for (final span in spans) {
          expect(span.range.start, lessThan(span.range.end));
          expect(span.range.end, lessThanOrEqualTo(text.length));
          expect(span.tokens, isNotEmpty);
        }
      },
    );

    test('math is drawn as an svg', () async {
      final inline = await KansoRenderer.mathSvg(
        source: 'x^2 + y^2',
        display: false,
        palette: palette,
      );
      expect(inline.markup, contains('<svg'));
      expect(inline.width, greaterThan(0));
      expect(inline.baseline, isNotNull);
      final block = await KansoRenderer.mathSvg(
        source: r'\frac{a}{b}',
        display: true,
        palette: palette,
      );
      expect(block.markup, contains('<svg'));
    });

    test('a diagram is drawn as an svg, and a broken one is refused', () async {
      final scene = await KansoRenderer.diagramSvg(
        kind: DiagramKind.excalidraw,
        source:
            '{"type":"excalidraw","version":2,"elements":[{"id":"a",'
            '"type":"rectangle","x":0,"y":0,"width":120,"height":60,'
            '"strokeColor":"#1e1e1e","backgroundColor":"transparent",'
            '"fillStyle":"solid","strokeWidth":1,"roughness":0,"seed":1}],'
            '"appState":{}}',
        palette: palette,
      );
      expect(scene.markup, contains('<svg'));
      await expectLater(
        KansoRenderer.diagramSvg(
          kind: DiagramKind.mermaid,
          source: 'this is not a diagram ::: [[',
          palette: palette,
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test('a note is previewed as a page in the palette', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'Preview',
        '# Heading\n\nA **bold** word.',
      );
      final html = await library.renderer().previewHtml(
        id: id,
        palette: palette,
      );
      expect(html, contains('<h1'));
      expect(html, contains('bold'));
    });

    test('the app page takes options, and says what an address is', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'In app',
        '- [ ] Water the beds\n\n```\nlet x = 1;\n```\n',
      );
      final html = await library.renderer().previewHtmlWith(
        id: id,
        palette: palette,
        options: const PreviewOptions(size: 18, utcOffsetSecs: 0, code: []),
      );
      expect(html, contains('kanso-task:0'));
      final renderer = library.renderer();
      expect(
        await renderer.linkTarget(noteId: id, url: 'kanso-task:0'),
        const PreviewLink.task(index: 0),
      );
      expect(
        await renderer.linkTarget(noteId: id, url: 'https://example.com/'),
        const PreviewLink.web(url: 'https://example.com/'),
      );
      expect(
        await renderer.linkTarget(noteId: id, url: 'javascript:alert(1)'),
        const PreviewLink.nothing(),
      );
      expect(
        await renderer.codeBlock(noteId: id, index: 0),
        contains('let x = 1;'),
      );
      await expectLater(
        renderer.codeBlock(noteId: id, index: 5),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test(
      'a note becomes the page the desktop exports, named for a file',
      () async {
        final library = await open();
        final id = await makeNote(
          library,
          'Exported: a page',
          '# Heading\n\nA [[Linked]] word, and `code`.\n',
        );
        final page = await library.renderer().pageHtml(
          id: id,
          palette: palette,
          options: const PreviewOptions(size: 16, utcOffsetSecs: 0, code: []),
        );
        expect(page.html, contains('<h1'));
        expect(page.html, contains('code'));
        // A page anybody can open carries no address only the app answers.
        expect(page.html, isNot(contains('kanso-task:')));
        expect(page.fileName, endsWith('.pdf'));
        expect(page.fileName, isNot(contains(':')));
      },
    );

    test('the diagram renderer is chosen, and read back', () async {
      final before = await KansoRenderer.diagramRenderer();
      await KansoRenderer.setDiagramRenderer(renderer: DiagramRenderer.kanso);
      expect(await KansoRenderer.diagramRenderer(), DiagramRenderer.kanso);
      await KansoRenderer.setDiagramRenderer(renderer: before);
      expect(await KansoRenderer.diagramRenderer(), before);
    });

    test('fonts are taken once', () async {
      expect(await KansoRenderer.setFonts(fonts: const []), isA<bool>());
    });
  });
}
