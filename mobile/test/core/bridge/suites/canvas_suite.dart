// Kanso — canvas_suite.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/canvas.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/core/bridge/api/library.dart';

import 'open_library.dart';

const InkStyle _pen = InkStyle(tool: 'pen', color: '#1e1e1e', width: 4);

/// A stroke along a line from `(x, y)`, `length` long, in canvas points.
List<PenSample> _line(double x, double y, [double length = 120]) => <PenSample>[
  for (var i = 0; i <= 12; i++)
    PenSample(x: x + length * i / 12, y: y, pressure: 0.5),
];

/// The canvases through the bridge: opening and making one, ink, erasers, a
/// lasso, cards, undo, saving and the exports. It runs on this computer
/// (`test/core/bridge`) and on a simulator or an emulator (`integration_test`).
void canvasSuite(OpenLibrary open) {
  group('KansoLibrary canvas', () {
    Future<(KansoLibrary, String, KansoCanvas)> fresh() async {
      final library = await open();
      final made = await library.createCanvas(title: 'Sketch');
      return (library, made.id, await library.canvas(id: made.id));
    }

    test('a new canvas is a canvas, empty, and opens', () async {
      final (library, id, canvas) = await fresh();
      expect(library.isCanvas(id: id), isTrue);
      final plain = await makeNote(library, 'A note', 'text');
      expect(library.isCanvas(id: plain), isFalse);
      final view = await canvas.view(known: const <String>[]);
      expect(view.title, 'Sketch');
      expect(view.strokes, isEmpty);
      expect(view.cards, isEmpty);
      expect(view.budget, CanvasBudget.ok);
    });

    test('a stroke is drawn as an outline, saved and reopened', () async {
      final (library, id, canvas) = await fresh();
      final drawn = await canvas.draw(
        style: _pen,
        samples: _line(10, 10),
        known: const <String>[],
      );
      expect(drawn.strokes, hasLength(1));
      expect(drawn.strokes.single.outline.length, greaterThan(20));
      expect(drawn.dirty, isTrue);
      expect(drawn.canUndo, isTrue);
      final saved = await canvas.save();
      expect(saved.bytes, greaterThan(0));
      final again = await (await library.canvas(
        id: id,
      )).view(known: const <String>[]);
      expect(again.strokes, hasLength(1));
      // A stroke the caller holds comes back without its outline.
      final known = await canvas.view(known: <String>[drawn.strokes.single.id]);
      expect(known.strokes.single.known, isTrue);
      expect(known.strokes.single.outline, isEmpty);
    });

    test('the live preview is an outline and stores nothing', () {
      final outline = canvasInkPreview(style: _pen, samples: _line(0, 0));
      expect(outline.length, greaterThan(20));
      expect(outline.length.isEven, isTrue);
    });

    test('the erasers, the lasso, a move, a delete and undo', () async {
      final (_, _, canvas) = await fresh();
      var view = await canvas.draw(
        style: _pen,
        samples: _line(0, 0),
        known: const <String>[],
      );
      view = await canvas.draw(
        style: _pen,
        samples: _line(0, 200),
        known: const <String>[],
      );
      expect(view.strokes, hasLength(2));

      // A point eraser and a dragged one take the stroke they touch.
      view = await canvas.eraseAt(
        x: 60,
        y: 0,
        radius: 10,
        known: const <String>[],
      );
      expect(view.strokes, hasLength(1));
      view = await canvas.undo(known: const <String>[]);
      expect(view.strokes, hasLength(2));
      view = await canvas.redo(known: const <String>[]);
      expect(view.strokes, hasLength(1));
      view = await canvas.undo(known: const <String>[]);
      view = await canvas.eraseStrokesAlong(
        path: const <CanvasPoint>[
          CanvasPoint(x: 0, y: 200),
          CanvasPoint(x: 120, y: 200),
        ],
        radius: 10,
        known: const <String>[],
      );
      expect(view.strokes, hasLength(1));
      view = await canvas.undo(known: const <String>[]);

      // The partial eraser cuts a gap in a stroke.
      view = await canvas.eraseAlong(
        path: const <CanvasPoint>[
          CanvasPoint(x: 50, y: 0),
          CanvasPoint(x: 70, y: 0),
        ],
        radius: 6,
        known: const <String>[],
      );
      expect(view.strokes.length, greaterThanOrEqualTo(2));
      view = await canvas.undo(known: const <String>[]);

      // A lasso holds what it circles; those strokes move and go.
      final held = await canvas.select(
        polygon: const <CanvasPoint>[
          CanvasPoint(x: -20, y: -20),
          CanvasPoint(x: 150, y: -20),
          CanvasPoint(x: 150, y: 20),
          CanvasPoint(x: -20, y: 20),
        ],
      );
      expect(held, hasLength(1));
      view = await canvas.moveStrokes(
        keys: held,
        dx: 0,
        dy: 50,
        known: const <String>[],
      );
      expect(view.strokes, hasLength(2));
      final now = await canvas.select(
        polygon: const <CanvasPoint>[
          CanvasPoint(x: -20, y: 30),
          CanvasPoint(x: 150, y: 30),
          CanvasPoint(x: 150, y: 70),
          CanvasPoint(x: -20, y: 70),
        ],
      );
      expect(now, hasLength(1));
      view = await canvas.deleteStrokes(keys: now, known: const <String>[]);
      expect(view.strokes, hasLength(1));
    });

    test('cards: text, a note, an arrow, edits and a delete', () async {
      final (library, _, canvas) = await fresh();
      final note = await makeNote(library, 'Linked', 'body');
      var view = await canvas.addTextCard(
        x: 0,
        y: 0,
        text: 'Hello',
        known: const <String>[],
      );
      view = await canvas.addNoteCard(
        noteId: note,
        x: 400,
        y: 0,
        known: const <String>[],
      );
      expect(view.cards, hasLength(2));
      final text = view.cards.firstWhere((c) => c.kind == 'text');
      final linked = view.cards.firstWhere((c) => c.noteId == note);
      // A note card shows the note's own title.
      expect(linked.text, 'Linked');
      view = await canvas.setCardText(
        id: text.id,
        text: 'Hello again',
        known: const <String>[],
      );
      expect(
        view.cards.firstWhere((c) => c.id == text.id).text,
        'Hello again',
      );
      view = await canvas.setCardRect(
        id: text.id,
        rect: const CanvasRect(x: 10, y: 20, width: 200, height: 100),
        known: const <String>[],
      );
      expect(view.cards.firstWhere((c) => c.id == text.id).rect.x, 10);
      view = await canvas.addArrow(
        from: text.id,
        to: linked.id,
        known: const <String>[],
      );
      expect(view.arrows, hasLength(1));
      view = await canvas.deleteCard(id: linked.id, known: const <String>[]);
      expect(view.cards, hasLength(1));
    });

    test('a private canvas has no room: the shared calls do nothing', () async {
      final (_, _, canvas) = await fresh();
      final view = await canvas.view(known: const <String>[]);
      expect(view.shared, isFalse);
      expect(view.readOnly, isFalse);
      final pulse = await canvas.pulse(known: const <String>[]);
      expect(pulse.view, isNull);
      expect(pulse.pointers, isEmpty);
      // Nothing streams, nothing is told, and there is nothing to take back.
      expect(
        await canvas.streamStroke(style: _pen, samples: _line(0, 0)),
        isFalse,
      );
      await canvas.cancelStroke();
      await canvas.setPointer(x: 4, y: 5, tool: 'pen');
      await canvas.clearPointer();
      expect(await canvas.pointers(), isEmpty);
      await canvas.reconnect();
      await canvas.close();
    });

    test('a selection moves to a canvas of its own', () async {
      final (library, id, canvas) = await fresh();
      await canvas.draw(
        style: _pen,
        samples: _line(0, 0),
        known: const <String>[],
      );
      final held = await canvas.select(
        polygon: const <CanvasPoint>[
          CanvasPoint(x: -20, y: -20),
          CanvasPoint(x: 150, y: -20),
          CanvasPoint(x: 150, y: 20),
          CanvasPoint(x: -20, y: 20),
        ],
      );
      final (made, view) = await canvas.moveToNewCanvas(
        keys: held,
        title: 'Part two',
        known: const <String>[],
      );
      expect(view.strokes, isEmpty);
      expect(made.id, isNot(id));
      expect(library.isCanvas(id: made.id), isTrue);
      final moved = await (await library.canvas(
        id: made.id,
      )).view(known: const <String>[]);
      expect(moved.strokes, hasLength(1));
    });

    test(
      'a sketch in a note: inserted, drawn, saved and opened as a canvas',
      () async {
        final library = await open();
        final note = await makeNote(library, 'Ideas', 'Hi');
        expect(library.takesSketches(noteId: note), isTrue);
        final canvas = await library.createCanvas(title: 'Board');
        expect(library.takesSketches(noteId: canvas.id), isFalse);

        final inserted = await library.sketchInsert(
          noteId: note,
          body: 'Hi',
          at: 2,
          width: 320,
          height: 240,
        );
        expect(inserted.body, startsWith('Hi'));
        expect(inserted.body, contains(inserted.id));
        expect(inserted.cursor, greaterThan(2));

        // The editor writes the body; the sketch then opens from the note.
        final withSketch = await makeNote(library, 'Sketched', inserted.body);
        final sketch = await library.sketch(
          noteId: withSketch,
          sketchId: inserted.id,
        );
        final info = sketch.sketchInfo();
        expect(info, isNotNull);
        expect(info!.id, inserted.id);
        expect(info.width, 320);
        expect(info.height, 240);
        expect(info.readOnly, isFalse);

        await sketch.draw(
          style: _pen,
          samples: _line(10, 10),
          known: const <String>[],
        );
        expect((await sketch.save()).bytes, greaterThan(0));
        final reopened = await library.sketch(
          noteId: withSketch,
          sketchId: inserted.id,
        );
        expect(
          (await reopened.view(known: const <String>[])).strokes,
          hasLength(1),
        );

        final copy = await reopened.openAsCanvas(title: 'From the sketch');
        expect(library.isCanvas(id: copy.id), isTrue);
        final view = await (await library.canvas(
          id: copy.id,
        )).view(known: const <String>[]);
        expect(view.strokes, hasLength(1));

        // A canvas is not a sketch.
        expect((await library.canvas(id: canvas.id)).sketchInfo(), isNull);
      },
    );

    test(
      'the exports are an SVG and an Excalidraw scene that imports',
      () async {
        final (library, _, canvas) = await fresh();
        await canvas.draw(
          style: _pen,
          samples: _line(0, 0),
          known: const <String>[],
        );
        final svg = await canvas.exportSvg(dark: false);
        expect(svg, contains('<svg'));
        final scene = await canvas.exportExcalidraw();
        expect(scene, contains('excalidraw'));
        final imported = await library.importExcalidraw(
          source: scene,
          title: 'Imported',
        );
        expect(library.isCanvas(id: imported.id), isTrue);
        final view = await (await library.canvas(
          id: imported.id,
        )).view(known: const <String>[]);
        expect(view.strokes, isNotEmpty);
      },
    );
  });
}
