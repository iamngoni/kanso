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

  /// Waits until the document has been asked for something: a call that
  /// changes no state is seen only there.
  Future<void> asked(bool Function() test) async {
    final until = DateTime.now().add(const Duration(seconds: 5));
    while (!test()) {
      if (DateTime.now().isAfter(until)) {
        fail('never asked');
      }
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
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

  test('an erase gesture is one call and writes nothing until idle', () async {
    await open();
    bloc.add(const CanvasEvent.strokeCommitted(_samples));
    await until(
      (s) => (s.view?.strokes.length ?? 0) == 1 && !(s.view?.dirty ?? true),
    );
    final saved = document.saves;
    final shown = bloc.state.edit;
    bloc.add(
      const CanvasEvent.erased(
        path: <CanvasPoint>[
          CanvasPoint(x: 10, y: 10),
          CanvasPoint(x: 20, y: 20),
          CanvasPoint(x: 30, y: 30),
        ],
        radius: 5,
        partial: false,
      ),
    );
    await until((s) => s.view?.strokes.isEmpty ?? false);
    expect(
      document.calls.where((c) => c == 'eraseStrokesAlong'),
      hasLength(1),
    );
    expect(bloc.state.edit, shown + 1);
    // Saved by the idle timer later, not by the erase itself.
    expect(document.saves, saved);
  });

  test('an erase that touches nothing shows and saves nothing', () async {
    await open();
    final shown = bloc.state.edit;
    bloc.add(
      const CanvasEvent.erased(
        path: <CanvasPoint>[CanvasPoint(x: 900, y: 900)],
        radius: 5,
        partial: false,
      ),
    );
    await _settle();
    await _settle();
    expect(document.calls, contains('eraseStrokesAlong'));
    expect(bloc.state.edit, shown);
    expect(document.saves, 0);
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

  group('a shared canvas', () {
    CanvasStroke remote() => fakeStroke('theirs', x: 200, y: 200);

    Future<void> openShared({bool readOnly = false}) async {
      document
        ..shared = true
        ..readOnly = readOnly;
      await open();
    }

    test('takes in what the others drew at its next tick', () async {
      await openShared();
      document
        ..remote = remote()
        ..remoteChanged = true;
      await until((s) => (s.view?.strokes.length ?? 0) == 1);
      expect(bloc.state.view?.strokes.single.id, 'theirs');
    });

    test('holds their edits back while a hand is down, then applies them '
        'when it lifts, though it only panned', () async {
      await openShared();
      bloc.add(const CanvasEvent.gestureChanged(active: true));
      document
        ..remote = remote()
        ..remoteChanged = true;
      bloc.add(const CanvasEvent.pulsed());
      await Future<void>.delayed(const Duration(milliseconds: 600));
      expect(bloc.state.view?.strokes, isEmpty);
      expect(document.calls, isNot(contains('pulse')));
      // The hand lifts: no stroke, no edit, only the end of a pan.
      bloc.add(const CanvasEvent.gestureChanged(active: false));
      await until((s) => (s.view?.strokes.length ?? 0) == 1);
    });

    test(
      'holds their edits back through a tap, and a stroke, the same',
      () async {
        await openShared();
        bloc.add(const CanvasEvent.gestureChanged(active: true));
        document
          ..remote = remote()
          ..remoteChanged = true;
        bloc
          ..add(const CanvasEvent.strokeCommitted(_samples))
          ..add(const CanvasEvent.pulsed());
        await Future<void>.delayed(const Duration(milliseconds: 600));
        // The stroke is the bloc's; the others' stays out of sight.
        expect(document.calls, contains('draw'));
        expect(document.calls, isNot(contains('pulse')));
        expect(
          bloc.state.view?.strokes.map((s) => s.id),
          isNot(contains('theirs')),
        );
        bloc.add(const CanvasEvent.gestureChanged(active: false));
        await until(
          (s) =>
              s.view?.strokes.any((stroke) => stroke.id == 'theirs') ?? false,
        );
        expect(bloc.state.view?.strokes, hasLength(2));
      },
    );

    test('hands the room an edit at once, not two seconds on', () async {
      await openShared();
      bloc.add(const CanvasEvent.textCardAdded(0, 0, 'Hello'));
      await asked(() => document.calls.contains('pulse'));
      expect(document.saves, 0, reason: 'the room writes it');
      await until((s) => !(s.view?.dirty ?? true));
    });

    test(
      'streams the stroke being drawn and takes it back when asked',
      () async {
        await openShared();
        bloc
          ..add(const CanvasEvent.strokeStreamed(_samples))
          ..add(const CanvasEvent.strokeCancelled());
        await asked(() => document.cancelled == 1);
        expect(document.streamed, hasLength(1));
        expect(document.streamed.single, _samples);
      },
    );

    test('a stroke someone erased under the pen is dropped, and their erase '
        'comes in when the hand lifts', () async {
      await openShared();
      bloc.add(const CanvasEvent.gestureChanged(active: true));
      document.erasedUnderPen = true;
      bloc.add(const CanvasEvent.strokeStreamed(_samples));
      await until((s) => s.erasedUnderPen == 1);
      document
        ..remote = remote()
        ..remoteChanged = true;
      bloc.add(const CanvasEvent.gestureChanged(active: false));
      await until((s) => (s.view?.strokes.length ?? 0) == 1);
    });

    test("shows the others' pointers, and keeps them moving through a "
        'gesture', () async {
      await openShared();
      const ada = CanvasPointerRow(
        accountId: 'a',
        name: 'Ada',
        x: 10,
        y: 20,
        tool: 'pen',
        colour: 0,
        colourLight: '#2F6AAE',
        colourDark: '#7FB0EC',
      );
      document.others = <CanvasPointerRow>[ada];
      await until((s) => s.pointers.length == 1);
      expect(bloc.state.pointers.single.name, 'Ada');
      bloc.add(const CanvasEvent.gestureChanged(active: true));
      document.others = <CanvasPointerRow>[];
      bloc.add(const CanvasEvent.pulsed());
      await until((s) => s.pointers.isEmpty);
      bloc.add(const CanvasEvent.gestureChanged(active: false));
    });

    test('tells the room where the pointer is, and when it leaves', () async {
      await openShared();
      bloc
        ..add(const CanvasEvent.toolChanged(CanvasTool.lasso))
        ..add(const CanvasEvent.pointerMoved(12, 34))
        ..add(const CanvasEvent.pointerLeft());
      await asked(() => document.clearedPointer == 1);
      expect(document.told, <(double, double, String)>[(12, 34, 'lasso')]);
    });

    test('a viewer draws, streams and edits nothing', () async {
      await openShared(readOnly: true);
      bloc
        ..add(const CanvasEvent.strokeCommitted(_samples))
        ..add(const CanvasEvent.strokeStreamed(_samples))
        ..add(const CanvasEvent.textCardAdded(0, 0, 'x'))
        ..add(const CanvasEvent.undone())
        ..add(
          const CanvasEvent.erased(
            path: <CanvasPoint>[CanvasPoint(x: 0, y: 0)],
            radius: 5,
            partial: false,
          ),
        );
      await _settle();
      await Future<void>.delayed(const Duration(milliseconds: 100));
      expect(document.streamed, isEmpty);
      expect(
        document.calls.where((c) => c != 'view' && c != 'pulse'),
        isEmpty,
      );
      expect(bloc.state.view?.readOnly, isTrue);
      // It still takes in what the others draw.
      document
        ..remote = remote()
        ..remoteChanged = true;
      await until((s) => (s.view?.strokes.length ?? 0) == 1);
    });

    test('joins the room again when the app comes back to the front', () async {
      await openShared();
      bloc.add(const CanvasEvent.resumed());
      await asked(() => document.reconnected == 1);
    });

    test('behind, the tick stops; back in front, it starts again', () async {
      await openShared();
      await asked(() => document.pulses > 0);
      bloc.add(const CanvasEvent.paused());
      await _settle();
      final paused = document.pulses;
      await Future<void>.delayed(canvasPulseEvery * 4);
      expect(document.pulses, paused, reason: 'no tick in the background');
      bloc.add(const CanvasEvent.resumed());
      await asked(() => document.reconnected == 1);
      await asked(() => document.pulses > paused + 1);
    });

    test('closing takes back, writes and leaves in one call', () async {
      await openShared();
      bloc.add(const CanvasEvent.textCardAdded(0, 0, 'Hello'));
      await asked(() => document.calls.contains('addTextCard'));
      await bloc.close();
      expect(document.closed, 1);
    });
  });

  group('a private canvas', () {
    test('has no tick and tells no room anything', () async {
      await open();
      bloc
        ..add(const CanvasEvent.pointerMoved(1, 2))
        ..add(const CanvasEvent.strokeStreamed(_samples))
        ..add(const CanvasEvent.strokeCancelled())
        ..add(const CanvasEvent.resumed());
      await Future<void>.delayed(canvasPulseEvery * 3);
      expect(document.pulses, 0);
      expect(document.told, isEmpty);
      expect(document.streamed, isEmpty);
      expect(document.cancelled, 0);
      expect(document.reconnected, 0);
      await bloc.close();
      expect(document.closed, 0);
    });
  });
}
