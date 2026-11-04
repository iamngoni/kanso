// Kanso — canvas_bloc_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/canvas/models/data/canvas_tool.dart';
import 'package:kanso/canvas/state/canvas/canvas_bloc.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/core/models/data/application_error.dart';

import '../../helpers/fake_canvas.dart';

const _samples = <PenSample>[
  PenSample(x: 10, y: 10, pressure: 0.5),
  PenSample(x: 30, y: 30, pressure: 0.5),
];

Future<void> _settle() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeCanvasDocument document;
  late CanvasBloc bloc;

  setUp(() {
    document = FakeCanvasDocument();
    bloc = CanvasBloc(FakeCanvasRepository(document: document));
  });

  tearDown(() async => bloc.close());

  /// Waits until the state is one [test] accepts, now or later.
  Future<void> until(bool Function(CanvasState state) test) async {
    if (test(bloc.state)) {
      return;
    }
    await bloc.stream.firstWhere(test).timeout(const Duration(seconds: 5));
  }

  Future<void> open() async {
    bloc.add(const CanvasEvent.started('canvas-1'));
    await until((s) => s.ready);
  }

  test('opens a canvas and shows its view', () async {
    await open();
    expect(bloc.state.status, CanvasStatus.ready);
    expect(bloc.state.view?.title, 'Sketch');
  });

  test('a locked canvas is shown as locked, with no view', () async {
    final locked = CanvasBloc(
      FakeCanvasRepository(
        openError: const ApplicationError.locked(message: 'Locked.'),
      ),
    )..add(const CanvasEvent.started('canvas-1'));
    await locked.stream.firstWhere((s) => s.status == CanvasStatus.locked);
    expect(locked.state.view, isNull);
    await locked.close();
  });

  test('any other failure to open is a failure', () async {
    final failed = CanvasBloc(
      FakeCanvasRepository(
        openError: const ApplicationError.notFound(message: 'Gone.'),
      ),
    )..add(const CanvasEvent.started('canvas-1'));
    await failed.stream.firstWhere((s) => s.status == CanvasStatus.failed);
    await failed.close();
  });

  test('a stroke is drawn and saved at once', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    await until((s) => !(s.view?.dirty ?? true));
    expect(document.calls, contains('draw'));
    expect(document.saves, 1);
    expect(bloc.state.view?.strokes, hasLength(1));
  });

  test('the pen in hand is the style a stroke is drawn with', () async {
    await open();
    bloc
      ..add(const CanvasEvent.toolChanged(CanvasTool.highlighter))
      ..add(const CanvasEvent.colorChanged('#e03131'))
      ..add(const CanvasEvent.widthChanged(9));
    await _settle();
    expect(bloc.state.tool, CanvasTool.highlighter);
    expect(bloc.state.color, '#e03131');
    expect(bloc.state.width, 9);
  });

  test('outlines already held are asked for without their outlines', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    await until((s) => (s.view?.strokes.length ?? 0) == 2);
    // The second draw was told about the first stroke...
    expect(document.knownSeen.last, isNotEmpty);
    // ...and the screen still holds its outline.
    expect(
      bloc.state.view!.strokes.every((s) => s.outline.isNotEmpty),
      isTrue,
    );
  });

  test('an erase, an undo and a redo change the view', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    bloc.add(const CanvasEvent.undone());
    await _settle();
    await until((s) => s.view?.strokes.isEmpty ?? false);
    expect(bloc.state.view?.canRedo, isTrue);
    bloc.add(const CanvasEvent.redone());
    await until((s) => (s.view?.strokes.length ?? 0) == 1);
    bloc.add(
      const CanvasEvent.erased(
        path: <CanvasPoint>[CanvasPoint(x: 10, y: 10)],
        radius: 5,
        partial: false,
      ),
    );
    await until((s) => s.view?.strokes.isEmpty ?? false);
    expect(document.calls, containsAll(<String>['undo', 'redo']));
    expect(document.calls, contains('eraseStrokesAlong'));
  });

  test('a partial erase uses the partial call', () async {
    await open();
    bloc.add(
      const CanvasEvent.erased(
        path: <CanvasPoint>[CanvasPoint(x: 1, y: 1)],
        radius: 5,
        partial: true,
      ),
    );
    await _settle();
    await _settle();
    expect(document.calls, contains('eraseAlong'));
  });

  test('the lasso holds strokes and a move selects the moved ones', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    bloc.add(const CanvasEvent.lassoClosed(<CanvasPoint>[]));
    await until((s) => s.selection.isNotEmpty);
    final before = bloc.state.selection.single.strokeId;
    bloc.add(const CanvasEvent.selectionMoved(50, 0));
    await until(
      (s) => s.selection.isNotEmpty && s.selection.single.strokeId != before,
    );
    bloc.add(const CanvasEvent.selectionCleared());
    await until((s) => s.selection.isEmpty);
  });

  test('a deleted selection leaves no strokes', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await _settle();
    bloc.add(const CanvasEvent.lassoClosed(<CanvasPoint>[]));
    await until((s) => s.selection.isNotEmpty);
    bloc.add(const CanvasEvent.selectionDeleted());
    await until((s) => s.view?.strokes.isEmpty ?? false);
    expect(bloc.state.selection, isEmpty);
  });

  test(
    'a pen seen turns fingers to panning and finger-draws toggles',
    () async {
      await open();
      bloc
        ..add(const CanvasEvent.stylusSeen())
        ..add(const CanvasEvent.fingerDrawsChanged(on: false));
      await _settle();
      expect(bloc.state.stylusSeen, isTrue);
      expect(bloc.state.fingerDraws, isFalse);
    },
  );

  test('a full canvas that refuses ink says so', () async {
    document
      ..budget = CanvasBudget.full
      ..drawError = const ApplicationError.invalid(message: 'Full.');
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await until((s) => s.problem != null);
    expect(bloc.state.problem, isNotNull);
    bloc.add(const CanvasEvent.messageDismissed());
    await until((s) => s.problem == null);
  });

  test('a save that fails is said so and keeps the canvas dirty', () async {
    document.saveError = const ApplicationError.storage(message: 'Disk.');
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await until((s) => s.problem == CanvasProblem.notSaved);
    expect(bloc.state.view?.dirty, isTrue);
  });

  test(
    'an edit that is not a stroke is written after two idle seconds',
    () async {
      await open();
      bloc.add(const CanvasEvent.textCardAdded(0, 0, 'Hello'));
      await until((s) => s.view?.dirty ?? false);
      await Future<void>.delayed(
        canvasIdleSave - const Duration(milliseconds: 500),
      );
      expect(document.saves, 0);
      await Future<void>.delayed(const Duration(milliseconds: 800));
      expect(document.saves, 1);
    },
  );

  test('closing writes what is left', () async {
    await open();
    bloc.add(const CanvasEvent.textCardAdded(0, 0, 'Hello'));
    await until((s) => s.view?.dirty ?? false);
    await bloc.close();
    expect(document.saves, 1);
  });
}
