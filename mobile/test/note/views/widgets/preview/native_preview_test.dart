// Kanso — native_preview_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:ui' show CheckedState, SemanticsAction;

import 'package:dartz/dartz.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show RenderParagraph, SelectedContent;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' hide Table, TableRow;
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/state/preview/preview_bloc.dart';
import 'package:kanso/note/views/widgets/note_body.dart';
import 'package:kanso/note/views/widgets/note_preview.dart';
import 'package:kanso/note/views/widgets/preview/native_preview.dart';
import 'package:kanso/note/views/widgets/preview/preview_blocks.dart';
import 'package:kanso/note/views/widgets/preview/preview_code.dart';
import 'package:kanso/note/views/widgets/preview/preview_list.dart';
import 'package:kanso/note/views/widgets/preview/preview_pdf_notes.dart';
import 'package:mocktail/mocktail.dart';

import '../../../../helpers/kanso_test_app.dart';
import '../../../../helpers/library_fixtures.dart';
import '../../../../helpers/mock_repositories.dart';
import '../../../../helpers/seeded_appearance_bloc.dart';
import '../../../../helpers/theme_tokens.dart';

/// A preview palette nothing else uses, so a colour found in the preview
/// can only have come from it.
const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFAFAFA,
  text: 0x111111,
  muted: 0x777777,
  accent: 0x1E66F5,
  border: 0xCCCCCC,
  surface: 0xEEEEEE,
  heading: 0x8839EF,
  quote: 0x40A02B,
  taskDone: 0x999999,
  inlineCodeBackground: 0xE0E0E0,
);

Color _rgb(int value) => Color(0xFF000000 | value);

PreviewInline _text(
  String text, {
  bool bold = false,
  bool italic = false,
  bool strike = false,
  bool code = false,
}) => PreviewInline.text(
  text: text,
  bold: bold,
  italic: italic,
  strike: strike,
  code: code,
);

PreviewBlock _paragraph(List<PreviewInline> inlines) =>
    PreviewBlock.paragraph(inlines: inlines);

const String _svg =
    '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" '
    'viewBox="0 0 40 20"><rect width="40" height="20" fill="#000"/></svg>';

/// The [TextSpan] whose text is [text] in the rich texts on screen.
TextSpan _span(WidgetTester tester, String text) {
  for (final rich in tester.widgetList<RichText>(find.byType(RichText))) {
    TextSpan? found;
    rich.text.visitChildren((span) {
      if (span is TextSpan && span.text == text) {
        found = span;
        return false;
      }
      return true;
    });
    if (found != null) {
      return found!;
    }
  }
  throw StateError('No span "$text"');
}

/// The style the span [text] is drawn in: its own over its parents'.
TextStyle _styleOf(WidgetTester tester, String text) {
  for (final rich in tester.widgetList<RichText>(find.byType(RichText))) {
    TextStyle? found;
    void walk(InlineSpan span, TextStyle inherited) {
      final style = inherited.merge(span.style);
      if (span is TextSpan) {
        if (span.text == text) {
          found = style;
        }
        for (final child in span.children ?? const <InlineSpan>[]) {
          walk(child, style);
        }
      }
    }

    walk(rich.text, const TextStyle());
    if (found != null) {
      return found!;
    }
  }
  throw StateError('No span "$text"');
}

void main() {
  late List<String> tapped;
  late int edits;

  setUp(() {
    tapped = <String>[];
    edits = 0;
  });

  Future<GlobalKey<NativePreviewState>> pump(
    WidgetTester tester,
    List<PreviewBlock> blocks, {
    double below = 0,
    GlobalKey<NativePreviewState>? previewKey,
    TextScaler? textScaler,
    ValueChanged<SelectedContent?>? onSelectionChanged,
    ScrollController? controller,
  }) async {
    final key = previewKey ?? GlobalKey<NativePreviewState>();
    await tester.pumpWidget(
      KansoTestApp(
        builder: textScaler == null
            ? null
            : (context, home) => MediaQuery(
                data: MediaQuery.of(context).copyWith(textScaler: textScaler),
                child: home,
              ),
        home: SelectionArea(
          onSelectionChanged: onSelectionChanged,
          child: CustomScrollView(
            controller: controller,
            slivers: <Widget>[
              NativePreview(
                key: key,
                blocks: blocks,
                palette: _palette,
                onLink: tapped.add,
                onEdit: () => edits++,
              ),
              SliverToBoxAdapter(child: SizedBox(height: below)),
            ],
          ),
        ),
      ),
    );
    await tester.pump();
    return key;
  }

  group('NativePreview', () {
    testWidgets('a paragraph keeps its bold, italic, struck and code runs', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[
          _text('plain '),
          _text('strong', bold: true),
          _text('slanted', italic: true),
          _text('gone', strike: true),
          _text('x = 1', code: true),
        ]),
      ]);
      expect(_styleOf(tester, 'strong').fontWeight, FontWeight.w700);
      expect(_styleOf(tester, 'slanted').fontStyle, FontStyle.italic);
      expect(
        _styleOf(tester, 'gone').decoration,
        TextDecoration.lineThrough,
      );
      expect(
        _styleOf(tester, 'x = 1').backgroundColor,
        _rgb(_palette.inlineCodeBackground),
      );
      expect(_styleOf(tester, 'plain ').color, _rgb(_palette.text));
    });

    testWidgets('a tap on a link hands its address over', (tester) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[
          _text('See '),
          PreviewInline.link(
            url: 'kanso-tag:work',
            inlines: <PreviewInline>[_text('work')],
          ),
        ]),
      ]);
      final recognizer = _span(tester, 'work').recognizer;
      expect(recognizer, isA<TapGestureRecognizer>());
      await tester.tapOnText(find.textRange.ofSubstring('work'));
      expect(tapped, <String>['kanso-tag:work']);
    });

    testWidgets(
      'headings, links and quotes are drawn in the preview palette',
      (tester) async {
        await pump(tester, <PreviewBlock>[
          PreviewBlock.heading(
            level: 2,
            id: 'plan',
            inlines: <PreviewInline>[_text('Plan')],
          ),
          _paragraph(<PreviewInline>[
            PreviewInline.link(
              url: 'https://example.com',
              inlines: <PreviewInline>[_text('a link')],
            ),
          ]),
          PreviewBlock.quote(
            blocks: <PreviewBlock>[
              _paragraph(<PreviewInline>[_text('quoted')]),
            ],
          ),
        ]);
        expect(_styleOf(tester, 'Plan').color, _rgb(_palette.heading!));
        expect(_styleOf(tester, 'a link').color, _rgb(_palette.accent));
        expect(_styleOf(tester, 'quoted').color, _rgb(_palette.quote));
      },
    );

    testWidgets('a task box asks to tick its task; one without a place '
        "can't be ticked", (tester) async {
      await pump(tester, <PreviewBlock>[
        PreviewBlock.list(
          ordered: false,
          start: 1,
          tight: true,
          items: <PreviewListItem>[
            PreviewListItem(
              task: const PreviewTaskBox(checked: false, index: 4),
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('Sweep')]),
              ],
            ),
            PreviewListItem(
              task: const PreviewTaskBox(checked: true),
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('Dust')]),
              ],
            ),
          ],
        ),
      ]);
      final boxes = find.byType(PreviewTaskCheckbox);
      expect(boxes, findsNWidgets(2));
      await tester.tap(boxes.first);
      await tester.tap(boxes.last);
      expect(tapped, <String>['kanso-task:4']);
      // A ticked task's text is in the done colour.
      expect(_styleOf(tester, 'Dust').color, _rgb(_palette.taskDone));
    });

    testWidgets('a code block shows its language and copies by its number', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.code(
          language: 'rust',
          runs: <PreviewCodeRun>[
            PreviewCodeRun(
              text: 'fn',
              color: 0xD20F39,
              bold: true,
              italic: false,
            ),
            PreviewCodeRun(text: ' main() {}', bold: false, italic: false),
          ],
          copy: 3,
        ),
      ]);
      expect(find.text('rust'), findsOneWidget);
      expect(_styleOf(tester, 'fn').color, _rgb(0xD20F39));
      final semantics = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Copy code'));
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:3']);
    });

    testWidgets('a heading can be scrolled to by its id', (tester) async {
      // Few enough lines that the heading is built; the long note's tests
      // scroll to headings that aren't.
      final key = await pump(tester, <PreviewBlock>[
        for (var at = 0; at < 12; at++)
          _paragraph(<PreviewInline>[_text('Line $at')]),
        PreviewBlock.heading(
          level: 1,
          id: 'end',
          inlines: <PreviewInline>[_text('The end')],
        ),
      ], below: 2000);
      final heading = key.currentState!.headingKey('end');
      expect(heading?.currentContext, isNotNull);
      final before = tester.getTopLeft(find.byKey(heading!)).dy;
      unawaited(key.currentState!.scrollTo('end'));
      await tester.pumpAndSettle();
      final after = tester.getTopLeft(find.byKey(heading)).dy;
      expect(after, lessThan(before));
      expect(after, closeTo(0, 1));
    });

    testWidgets('a figure is its SVG', (tester) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.figure(
          kind: PreviewFigureKind.diagram,
          svg: PreviewSvg(markup: _svg, width: 40, height: 20),
        ),
      ]);
      expect(find.byType(SvgPicture), findsOneWidget);
      expect(tester.getSize(find.byType(SvgPicture)), const Size(40, 20));
    });

    testWidgets('a PDF block is a quiet line, not its data', (tester) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.pdfNotes(
          url: 'inkdrop://file:abc123',
          summary: '4 highlights · 1 comment',
        ),
      ]);
      expect(find.text('4 highlights · 1 comment'), findsOneWidget);
      // Without a note to open the list for, it only reads.
      expect(find.byType(PreviewPdfNotes), findsOneWidget);
      await tester.tap(find.text('4 highlights · 1 comment'));
      await tester.pump();
      expect(tapped, isEmpty);
    });

    testWidgets('a table shows its header and cells', (tester) async {
      await pump(tester, <PreviewBlock>[
        PreviewBlock.table(
          aligns: const <PreviewAlign>[PreviewAlign.left, PreviewAlign.right],
          head: <PreviewCell>[
            PreviewCell(inlines: <PreviewInline>[_text('Fruit')]),
            PreviewCell(inlines: <PreviewInline>[_text('Count')]),
          ],
          rows: <PreviewRow>[
            PreviewRow(
              cells: <PreviewCell>[
                PreviewCell(inlines: <PreviewInline>[_text('Apple')]),
                PreviewCell(inlines: <PreviewInline>[_text('3')]),
              ],
            ),
          ],
        ),
      ]);
      expect(find.byType(Table), findsOneWidget);
      for (final text in <String>['Fruit', 'Count', 'Apple', '3']) {
        expect(find.text(text, findRichText: true), findsOneWidget);
      }
      expect(_styleOf(tester, 'Fruit').fontWeight, FontWeight.w600);
    });

    testWidgets("a query view's title opens its note", (tester) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.query(
          columns: <String>['Title', 'Status'],
          rows: <PreviewQueryRow>[
            PreviewQueryRow(
              url: 'inkdrop://note/a',
              title: 'Alpha',
              cells: <String>['active'],
            ),
          ],
        ),
        const PreviewBlock.query(
          columns: <String>['Title'],
          rows: <PreviewQueryRow>[],
        ),
      ]);
      expect(find.text('Status'), findsOneWidget);
      expect(find.text('active'), findsOneWidget);
      expect(find.text('No notes match.'), findsOneWidget);
      await tester.tap(find.text('Alpha'));
      expect(tapped, <String>['inkdrop://note/a']);
    });

    testWidgets('a source that failed shows its badge and its code', (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.failed(
          badge: 'Unsupported diagram',
          detail: 'Kanso does not draw gantt.',
          language: 'gantt',
          code: 'gantt\n  title Plan',
          copy: 1,
        ),
        const PreviewBlock.queryInvalid(
          code: 'where nonsense',
          message: 'Unknown field.',
        ),
      ]);
      expect(find.text('Unsupported diagram'), findsOneWidget);
      expect(find.text('gantt\n  title Plan'), findsOneWidget);
      expect(find.text('gantt'), findsOneWidget);
      expect(find.text('Query'), findsOneWidget);
      expect(find.text('Unknown field.'), findsOneWidget);
      final semantics = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Copy code'));
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:1']);
    });

    testWidgets('lists nested as deep as the core allows fit a narrow '
        'screen', (tester) async {
      tester.view
        ..physicalSize = const Size(320, 2400)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      var list = PreviewBlock.list(
        ordered: false,
        start: 1,
        tight: true,
        items: <PreviewListItem>[
          PreviewListItem(
            blocks: <PreviewBlock>[
              _paragraph(<PreviewInline>[_text('deepest')]),
            ],
          ),
        ],
      );
      for (var depth = 63; depth > 0; depth--) {
        list = PreviewBlock.list(
          ordered: depth.isEven,
          start: 1,
          tight: depth % 3 != 0,
          items: <PreviewListItem>[
            PreviewListItem(
              task: depth % 5 == 0
                  ? PreviewTaskBox(checked: depth.isEven, index: depth)
                  : null,
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('level $depth')]),
                list,
              ],
            ),
          ],
        );
      }
      await pump(tester, <PreviewBlock>[list]);
      expect(tester.takeException(), isNull);
      final deepest = tester.getRect(find.text('deepest', findRichText: true));
      expect(deepest.left, greaterThanOrEqualTo(0));
      expect(deepest.right, lessThanOrEqualTo(320.5));
      expect(deepest.width, greaterThan(100));
      // The boxes deep down can still be ticked.
      await tester.tap(find.byType(PreviewTaskCheckbox).last);
      expect(tapped, <String>['kanso-task:60']);
    });

    testWidgets('a screen reader can copy code and tick a task', (
      tester,
    ) async {
      final semantics = tester.ensureSemantics();
      await pump(tester, <PreviewBlock>[
        const PreviewBlock.code(
          runs: <PreviewCodeRun>[
            PreviewCodeRun(text: 'x', bold: false, italic: false),
          ],
          copy: 3,
        ),
        PreviewBlock.list(
          ordered: false,
          start: 1,
          tight: true,
          items: <PreviewListItem>[
            PreviewListItem(
              task: const PreviewTaskBox(checked: false, index: 4),
              blocks: <PreviewBlock>[
                _paragraph(<PreviewInline>[_text('Sweep')]),
              ],
            ),
          ],
        ),
      ]);
      final copy = tester.semantics.find(find.bySemanticsLabel('Copy code'));
      expect(copy.getSemanticsData().hasAction(SemanticsAction.tap), isTrue);
      tester.semantics.tap(find.semantics.byLabel('Copy code'));
      final task = tester.semantics.find(find.bySemanticsLabel('Task'));
      final data = task.getSemanticsData();
      expect(data.hasAction(SemanticsAction.tap), isTrue);
      expect(data.flagsCollection.isChecked, CheckedState.isFalse);
      tester.semantics.tap(find.semantics.byLabel('Task'));
      await tester.pump();
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:3', 'kanso-task:4']);
    });

    testWidgets('headings without an id, or with the same one, are drawn; '
        'the first with an id is the one scrolled to', (tester) async {
      PreviewBlock heading(String id, String text) => PreviewBlock.heading(
        level: 2,
        id: id,
        inlines: <PreviewInline>[_text(text)],
      );
      final key = await pump(tester, <PreviewBlock>[
        heading('', 'Untitled'),
        heading('', 'Untitled'),
        heading('again', 'Again'),
        heading('again', 'Again'),
      ]);
      expect(tester.takeException(), isNull);
      expect(find.text('Untitled', findRichText: true), findsNWidgets(2));
      expect(key.currentState!.headingKey(''), isNull);
      final again = key.currentState!.headingKey('again');
      expect(again?.currentContext, isNotNull);
      final both = find.text('Again', findRichText: true);
      expect(both, findsNWidgets(2));
      expect(
        tester.getTopLeft(find.byKey(again!)).dy,
        closeTo(tester.getTopLeft(both.first).dy, 0.5),
      );
      // Drawn again, the same headings keep their key.
      await pump(tester, <PreviewBlock>[
        heading('', 'Untitled'),
        heading('again', 'Again'),
        heading('again', 'Again'),
      ], previewKey: key);
      expect(tester.takeException(), isNull);
      expect(key.currentState!.headingKey('again'), same(again));
    });

    testWidgets('a link held while its text is drawn again still answers', (
      tester,
    ) async {
      List<PreviewBlock> blocks() => <PreviewBlock>[
        _paragraph(<PreviewInline>[
          PreviewInline.link(
            url: 'kanso-tag:work',
            inlines: <PreviewInline>[_text('work')],
          ),
        ]),
      ];
      final key = await pump(tester, blocks());
      final at =
          tester.getTopLeft(find.text('work', findRichText: true)) +
          const Offset(4, 8);
      final gesture = await tester.startGesture(at);
      await pump(tester, blocks(), previewKey: key);
      await gesture.up();
      await tester.pump();
      expect(tapped, <String>['kanso-tag:work']);
    });

    testWidgets("math and figures grow with the system's text, figures "
        'within the width', (tester) async {
      const math = PreviewSvg(
        markup: _svg,
        width: 40,
        height: 20,
        baseline: 15,
      );
      Future<(Size, Size, Size)> sizes(double scale) async {
        await pump(tester, <PreviewBlock>[
          _paragraph(<PreviewInline>[
            _text('x '),
            const PreviewInline.math(svg: math),
          ]),
          const PreviewBlock.figure(kind: PreviewFigureKind.math, svg: math),
          const PreviewBlock.figure(
            kind: PreviewFigureKind.diagram,
            svg: PreviewSvg(markup: _svg, width: 4000, height: 2000),
          ),
        ], textScaler: TextScaler.linear(scale));
        final svgs = find.byType(SvgPicture);
        return (
          tester.getRect(svgs.at(0)).size,
          tester.getRect(svgs.at(1)).size,
          tester.getRect(svgs.at(2)).size,
        );
      }

      final (inline1, figure1, wide1) = await sizes(1);
      final (inline2, figure2, wide2) = await sizes(2);
      expect(inline1, const Size(40, 20));
      expect(inline2, const Size(80, 40));
      expect(figure1, const Size(40, 20));
      expect(figure2, const Size(80, 40));
      final width = tester.getSize(find.byType(CustomScrollView)).width;
      expect(wide1.width, closeTo(width, 0.5));
      expect(wide2.width, closeTo(width, 0.5));
    });

    testWidgets("what is selected is drawn in the preview's accent", (
      tester,
    ) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('Some text to select')]),
      ]);
      final text = find.text('Some text to select', findRichText: true);
      final accent = _rgb(_palette.accent);
      final style = DefaultSelectionStyle.of(tester.element(text));
      expect(style.selectionColor, accent.withValues(alpha: 0.3));
      expect(style.cursorColor, accent);
      expect(
        tester.renderObject<RenderParagraph>(text).selectionColor,
        accent.withValues(alpha: 0.3),
      );
    });

    testWidgets('a double tap asks to edit', (tester) async {
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('Some text to read')]),
      ]);
      final at = tester.getCenter(
        find.text('Some text to read', findRichText: true),
      );
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(at);
      await tester.pumpAndSettle();
      expect(edits, 1);
    });
  });

  group('NativePreview, a long note', () {
    PreviewBlock heading(String id, String text) => PreviewBlock.heading(
      level: 2,
      id: id,
      inlines: <PreviewInline>[_text(text)],
    );

    /// 5,000 blocks: a heading `h<n>` every tenth, paragraphs between; the
    /// paragraph at [changedAt] says "Changed".
    List<PreviewBlock> long({int? changedAt}) => <PreviewBlock>[
      for (var at = 0; at < 5000; at++)
        if (at % 10 == 0)
          heading('h$at', 'Heading $at')
        else
          _paragraph(<PreviewInline>[
            _text(at == changedAt ? 'Changed' : 'Paragraph $at, some words'),
          ]),
    ];

    testWidgets('builds only the blocks near the screen, quickly', (
      tester,
    ) async {
      final watch = Stopwatch()..start();
      await pump(tester, long());
      watch.stop();
      final built = find
          .byType(PreviewBlockView, skipOffstage: false)
          .evaluate()
          .length;
      expect(built, greaterThan(5));
      expect(built, lessThan(100));
      expect(
        find.text('Paragraph 4999, some words', findRichText: true),
        findsNothing,
      );
      // Generous: drawing all 5,000 blocks took over ten seconds.
      expect(watch.elapsed, lessThan(const Duration(seconds: 5)));
    });

    testWidgets('a change to one block builds only that block again', (
      tester,
    ) async {
      final key = await pump(tester, long());
      NativePreview.debugBlockBuilds = 0;
      final watch = Stopwatch()..start();
      await pump(tester, long(changedAt: 3), previewKey: key);
      watch.stop();
      expect(NativePreview.debugBlockBuilds, 1);
      expect(find.text('Changed', findRichText: true), findsOneWidget);
      expect(watch.elapsed, lessThan(const Duration(seconds: 2)));
    });

    testWidgets('a heading not built yet can be scrolled to, the last too', (
      tester,
    ) async {
      final key = await pump(tester, long());
      final state = key.currentState!;
      final screen = tester.view.physicalSize / tester.view.devicePixelRatio;
      for (final anchor in <String>['h2500', 'h4990', 'h10']) {
        final heading = state.headingKey(anchor)!;
        if (anchor != 'h10') {
          expect(heading.currentContext, isNull);
        }
        unawaited(state.scrollTo(anchor));
        await tester.pumpAndSettle();
        expect(heading.currentContext, isNotNull, reason: anchor);
        final top = tester.getTopLeft(find.byKey(heading)).dy;
        expect(top, greaterThanOrEqualTo(-0.5), reason: anchor);
        expect(top, lessThan(screen.height), reason: anchor);
        if (anchor != 'h4990') {
          expect(top, closeTo(0, 1), reason: anchor);
        }
      }
    });

    testWidgets('the last heading is scrolled to at once, and the first '
        'again', (tester) async {
      // Room below, so the last heading can come to the top.
      final key = await pump(tester, long(), below: 1000);
      final state = key.currentState!;
      Future<Duration> scrollTo(String anchor) async {
        final watch = Stopwatch()..start();
        unawaited(state.scrollTo(anchor));
        await tester.pumpAndSettle();
        watch.stop();
        final heading = state.headingKey(anchor)!;
        expect(heading.currentContext, isNotNull, reason: anchor);
        expect(
          tester.getTopLeft(find.byKey(heading)).dy,
          closeTo(0, 1),
          reason: anchor,
        );
        return watch.elapsed;
      }

      final down = await scrollTo('h4990');
      final up = await scrollTo('h0');
      debugPrint('Scrolled to h4990 in $down, back to h0 in $up');
      // Laying out every block on the way took three to five seconds.
      expect(down, lessThan(const Duration(seconds: 1)));
      expect(up, lessThan(const Duration(seconds: 1)));
    });

    testWidgets('a jump far down builds only the blocks where it lands', (
      tester,
    ) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await pump(tester, long(), controller: controller);
      NativePreview.debugBlockBuilds = 0;
      final watch = Stopwatch()..start();
      controller.jumpTo(controller.position.maxScrollExtent / 2);
      await tester.pump();
      watch.stop();
      debugPrint(
        'Jumped half way down in ${watch.elapsed}, building '
        '${NativePreview.debugBlockBuilds} blocks',
      );
      expect(NativePreview.debugBlockBuilds, greaterThan(0));
      expect(NativePreview.debugBlockBuilds, lessThan(100));
      expect(
        find.text('Paragraph 1, some words', findRichText: true),
        findsNothing,
      );
      final shown = find.textContaining(
        RegExp('^Paragraph [0-9]+,'),
        findRichText: true,
      );
      expect(shown, findsWidgets);
      final first = tester.widget<RichText>(shown.first).text.toPlainText();
      final at = int.parse(RegExp('[0-9]+').firstMatch(first)![0]!);
      expect(at, greaterThan(1000));
      expect(at, lessThan(4000));
    });

    testWidgets('a heading that becomes a repeat loses its key safely', (
      tester,
    ) async {
      final key = await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('Intro')]),
        heading('same', 'Second'),
      ]);
      final first = key.currentState!.headingKey('same');
      await pump(tester, <PreviewBlock>[
        heading('same', 'First'),
        heading('same', 'Second'),
      ], previewKey: key);
      expect(tester.takeException(), isNull);
      expect(key.currentState!.headingKey('same'), same(first));
      expect(
        tester.getTopLeft(find.byKey(first!)).dy,
        closeTo(
          tester.getTopLeft(find.text('First', findRichText: true)).dy,
          0.5,
        ),
      );
    });

    testWidgets('text can be selected across two blocks', (tester) async {
      SelectedContent? selected;
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('First block')]),
        _paragraph(<PreviewInline>[_text('Second block')]),
      ], onSelectionChanged: (content) => selected = content);
      final from = tester.getTopLeft(
        find.text('First block', findRichText: true),
      );
      final to = tester.getBottomRight(
        find.text('Second block', findRichText: true),
      );
      final gesture = await tester.startGesture(
        from + const Offset(1, 4),
        kind: PointerDeviceKind.mouse,
      );
      await tester.pump();
      await gesture.moveTo(to - const Offset(1, 4));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(selected?.plainText, contains('First block'));
      expect(selected?.plainText, contains('Second block'));
    });
  });

  group('NativePreview, a long code block', () {
    /// A code block of [lines] lines, `let aN = N;`, its first line wide.
    PreviewBlock code(int lines) => PreviewBlock.code(
      language: 'rust',
      runs: <PreviewCodeRun>[
        PreviewCodeRun(
          text: '// ${'wide ' * 60}\n',
          color: 0x8C8FA1,
          bold: false,
          italic: true,
        ),
        for (var line = 1; line < lines; line++) ...<PreviewCodeRun>[
          const PreviewCodeRun(text: 'let', bold: true, italic: false),
          PreviewCodeRun(
            text: ' a$line = $line;${line + 1 < lines ? '\n' : ''}',
            bold: false,
            italic: false,
          ),
        ],
      ],
      copy: 7,
    );

    final chunks = find.byType(PreviewCodeChunk);

    /// Scrolls the note so that the first two pieces meet mid-screen.
    Future<void> showJoin(
      WidgetTester tester,
      ScrollController controller,
    ) async {
      final join = tester.getBottomLeft(chunks.first).dy;
      controller.jumpTo(controller.offset + join - 300);
      await tester.pump();
      // The pieces are widened to the widest after the frame they are laid
      // out in.
      await tester.pump();
      expect(chunks, findsAtLeastNWidgets(2));
    }

    testWidgets('is built a piece at a time, as one box that copies whole', (
      tester,
    ) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await pump(tester, <PreviewBlock>[
        _paragraph(<PreviewInline>[_text('Before')]),
        code(1000),
        _paragraph(<PreviewInline>[_text('After')]),
      ], controller: controller);
      final built = find.byType(PreviewCodeChunk, skipOffstage: false);
      expect(built, findsAtLeastNWidgets(1));
      expect(built.evaluate().length, lessThanOrEqualTo(3));
      expect(find.text('After', findRichText: true), findsNothing);
      expect(find.text('rust'), findsOneWidget);
      final semantics = tester.ensureSemantics();
      expect(find.bySemanticsLabel('Copy code'), findsOneWidget);
      await tester.tap(find.bySemanticsLabel('Copy code'));
      semantics.dispose();
      expect(tapped, <String>['kanso-copy:7']);

      await showJoin(tester, controller);
      final first = tester.getRect(chunks.at(0));
      final second = tester.getRect(chunks.at(1));
      expect(second.top, closeTo(first.bottom, 0.01));
      expect(second.left, first.left);
      expect(second.width, first.width);
      // Only the first piece has the language and the copy button; only
      // the first has a top edge, and neither a bottom one.
      final boxes = tester
          .widgetList<PreviewCodeBox>(find.byType(PreviewCodeBox))
          .toList();
      expect(boxes[0].top, isTrue);
      expect(boxes[0].bottom, isFalse);
      expect(boxes[0].copy, 7);
      expect(boxes[1].top, isFalse);
      expect(boxes[1].bottom, isFalse);
      expect(boxes[1].copy, isNull);
      expect(boxes[1].language, isNull);
      expect(tester.takeException(), isNull);
    });

    testWidgets('its last piece closes the box, and its lines are all there', (
      tester,
    ) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await pump(tester, <PreviewBlock>[code(260)], controller: controller);
      // The heights guessed are measured as the pieces are laid out, so the
      // end moves until it is reached.
      final end = find.textContaining('a259 = 259;', findRichText: true);
      for (var tries = 0; tries < 5 && end.evaluate().isEmpty; tries++) {
        controller.jumpTo(controller.position.maxScrollExtent);
        await tester.pumpAndSettle();
      }
      final last = tester
          .widgetList<PreviewCodeBox>(find.byType(PreviewCodeBox))
          .last;
      expect(last.bottom, isTrue);
      expect(last.top, isFalse);
      // 260 lines: five pieces of 50 and one of 10.
      expect(end, findsOneWidget);
      final lines = tester.widget<RichText>(end).text.toPlainText();
      expect(lines.split('\n'), hasLength(10));
      expect(lines, startsWith('let a250 = 250;'));
    });

    testWidgets('its pieces scroll sideways together', (tester) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);
      await pump(tester, <PreviewBlock>[code(1000)], controller: controller);
      await showJoin(tester, controller);
      ScrollPosition sideways(int at) => tester
          .state<ScrollableState>(
            find.descendant(
              of: chunks.at(at),
              matching: find.byType(Scrollable),
            ),
          )
          .position;
      // The second piece's lines are short, but it scrolls as far as the
      // first's wide line.
      expect(
        sideways(1).maxScrollExtent,
        closeTo(sideways(0).maxScrollExtent, 0.5),
      );
      expect(sideways(1).maxScrollExtent, greaterThan(100));
      // Either side of where the pieces meet, mid-screen.
      final join = tester.getBottomLeft(chunks.first) + const Offset(200, 0);
      await tester.dragFrom(join + const Offset(0, 100), const Offset(-200, 0));
      await tester.pumpAndSettle();
      expect(sideways(1).pixels, greaterThan(100));
      expect(sideways(0).pixels, sideways(1).pixels);
      await tester.dragFrom(join - const Offset(0, 100), const Offset(80, 0));
      await tester.pumpAndSettle();
      expect(sideways(0).pixels, lessThan(sideways(1).maxScrollExtent - 100));
      expect(sideways(1).pixels, sideways(0).pixels);
    });

    testWidgets('selected across two pieces, its lines keep their break', (
      tester,
    ) async {
      final controller = ScrollController();
      addTearDown(controller.dispose);
      SelectedContent? selected;
      await pump(
        tester,
        <PreviewBlock>[code(1000)],
        controller: controller,
        onSelectionChanged: (content) => selected = content,
      );
      await showJoin(tester, controller);
      // From the last line of the first piece to the first of the second.
      final join = tester.getBottomLeft(chunks.first);
      final gesture = await tester.startGesture(
        join + const Offset(1 + 16 + 30, -10),
        kind: PointerDeviceKind.mouse,
      );
      await tester.pump();
      await gesture.moveTo(join + const Offset(1 + 16 + 120, 10));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(selected?.plainText, contains('a49 = 49;\nlet a5'));
    });
  });

  group('NotePreview', () {
    late MockLibraryRepository library;
    late MockRendererRepository renderer;
    late MockThemesRepository themes;

    setUpAll(() {
      registerFallbackValue(_palette);
      registerFallbackValue(
        const PreviewOptions(size: 16, utcOffsetSecs: 0, code: []),
      );
    });

    setUp(() {
      library = MockLibraryRepository();
      renderer = MockRendererRepository();
      themes = MockThemesRepository();
      when(
        () => library.changes(),
      ).thenAnswer((_) => const Stream<List<String>>.empty());
      when(() => library.note(any())).thenAnswer(
        (_) async => Right<ApplicationError, NoteView>(mockOpenNote),
      );
      when(() => themes.fit(any(), any(), any())).thenAnswer(
        (_) async => Right<ApplicationError, ThemeTokens>(
          tokensFor(fakePalette('paper')),
        ),
      );
      when(() => renderer.previewBlocks(any(), any(), any())).thenAnswer(
        (_) async => Right<ApplicationError, List<PreviewBlock>>(
          <PreviewBlock>[
            _paragraph(<PreviewInline>[_text('Drawn natively')]),
          ],
        ),
      );
      getIt.registerFactory<PreviewBloc>(
        () => PreviewBloc(
          library,
          renderer,
          themes,
          MockWebLinkRepository(),
          MockUndoNoticeBloc(),
          MockHapticsRepository(),
        ),
      );
      addTearDown(() => getIt.unregister<PreviewBloc>());
    });

    testWidgets(
      'draws the note in the chosen preview theme, fitted with the rest',
      (tester) async {
        final appearance = SeededAppearanceBloc(
          const AppearanceState(
            selection: ThemeSelection(
              followSystem: false,
              syntax: 'gallery-dracula',
              preview: 'gallery-paper',
            ),
          ),
        );
        addTearDown(appearance.close);
        await tester.pumpWidget(
          KansoTestApp(
            brightness: Brightness.dark,
            home: BlocProvider<AppearanceBloc>.value(
              value: appearance,
              child: CustomScrollView(
                slivers: <Widget>[
                  NotePreview(note: mockOpenNote, lines: const []),
                ],
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        verify(
          () => themes.fit('kanso-dark', 'gallery-dracula', 'gallery-paper'),
        ).called(1);
        expect(find.byType(NativePreview), findsOneWidget);
        expect(
          find.text('Drawn natively', findRichText: true),
          findsOneWidget,
        );
        verifyNever(() => renderer.builtinThemeTokens(any()));
      },
    );
  });

  group('NoteBody', () {
    testWidgets("selects in the preview theme's accent, handles too", (
      tester,
    ) async {
      final appearance = SeededAppearanceBloc(
        AppearanceState(
          light: KansoPalette.kansoLight.copyWith(preview: _palette),
        ),
      );
      addTearDown(appearance.close);
      await tester.pumpWidget(
        KansoTestApp(
          home: BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: NoteBody(
              note: mockOpenNote,
              lines: const [],
              index: const LibraryIndexState(),
              gutter: 16,
            ),
          ),
        ),
      );
      await tester.pump();
      final accent = _rgb(_palette.accent);
      // Not the app's own accent, which a selection would otherwise be in.
      expect(KansoPalette.kansoLight.colors.accent, isNot(accent));
      final region = tester.element(find.byType(SelectableRegion));
      expect(TextSelectionTheme.of(region).selectionHandleColor, accent);
      expect(
        DefaultSelectionStyle.of(region).selectionColor,
        accent.withValues(alpha: 0.3),
      );
      final title = find.text(mockOpenNote.title, findRichText: true);
      expect(
        tester.renderObject<RenderParagraph>(title.first).selectionColor,
        accent.withValues(alpha: 0.3),
      );
    });
  });
}
