// Kanso — canvas_bloc.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/data/canvas_tool.dart';
import '../../models/repos/abs/canvas_repository.dart';

part 'canvas_bloc.freezed.dart';
part 'canvas_event.dart';
part 'canvas_state.dart';

/// How long after the last edit that isn't a stroke the canvas is written.
const Duration canvasIdleSave = Duration(seconds: 2);

/// One canvas open for drawing. The document lives in Rust; this holds the
/// view the screen paints, the tool, and the rule for saving: a stroke is
/// written when it ends, anything else two seconds after the last edit, and
/// whatever is left when the screen closes. One transformer carries every
/// event, so a stroke, an erase and an undo never overlap.
@injectable
class CanvasBloc extends Bloc<CanvasEvent, CanvasState> {
  CanvasBloc(this._repository) : super(const CanvasState()) {
    on<CanvasEvent>(_onEvent, transformer: sequential());
  }

  final CanvasRepository _repository;
  CanvasDocument? _document;
  Timer? _idle;

  /// The outlines the screen already holds, by stroke ID.
  final Map<String, CanvasStroke> _held = <String, CanvasStroke>{};

  /// The document, for exports. Null until the canvas opened.
  CanvasDocument? get document => _document;

  Future<void> _onEvent(CanvasEvent event, Emitter<CanvasState> emit) async {
    switch (event) {
      case CanvasStartedEvent(:final noteId):
        await _open(noteId, emit);
      case CanvasSketchStartedEvent(:final noteId, :final sketchId):
        await _open(noteId, emit, sketchId: sketchId);
      case CanvasStylusSeenEvent():
        if (!state.stylusSeen) {
          emit(state.copyWith(stylusSeen: true));
        }
      case CanvasToolChangedEvent(:final tool):
        emit(
          state.copyWith(
            tool: tool,
            selection: tool == CanvasTool.lasso
                ? state.selection
                : const <StrokeKey>[],
          ),
        );
      case CanvasColorChangedEvent(:final color):
        emit(state.copyWith(color: color));
      case CanvasWidthChangedEvent(:final width):
        emit(state.copyWith(width: width));
      case CanvasFingerDrawsChangedEvent(:final on):
        emit(state.copyWith(fingerDraws: on));
      case CanvasStrokeCommittedEvent(:final samples):
        final style = InkStyle(
          tool: state.tool.draws ? state.tool.ink : 'pen',
          color: state.color,
          width: state.width,
        );
        await _change(
          emit,
          (doc) => doc.draw(style, samples, _known()),
          // A canvas is written when a stroke ends; a sketch two seconds
          // later, so a continuous sketch rewrites its note's text at most
          // every two seconds.
          saveNow: !_sketch,
        );
      case CanvasErasedEvent(:final path, :final radius, :final partial):
        await _change(
          emit,
          (doc) => partial
              ? doc.eraseAlong(path, radius, _known())
              : doc.eraseStrokesAlong(path, radius, _known()),
          quietIfUnchanged: true,
        );
      case CanvasLassoClosedEvent(:final polygon):
        final doc = _document;
        if (doc != null) {
          final held = await doc.select(polygon);
          emit(
            state.copyWith(
              selection: held.getOrElse(() => const <StrokeKey>[]),
            ),
          );
        }
      case CanvasSelectionClearedEvent():
        emit(state.copyWith(selection: const <StrokeKey>[]));
      case CanvasSelectionDeletedEvent():
        final keys = state.selection;
        await _change(emit, (doc) => doc.deleteStrokes(keys, _known()));
      case CanvasSelectionMovedEvent(:final dx, :final dy):
        final keys = state.selection;
        // Moved strokes are new ones: the selection is what was moved.
        await _change(
          emit,
          (doc) => doc.moveStrokes(keys, dx, dy, _known()),
          selectBorn: true,
        );
      case CanvasSelectionMovedToNewCanvasEvent(:final title):
        final keys = state.selection;
        final doc = _document;
        if (doc != null && keys.isNotEmpty) {
          final moved = await doc.moveToNewCanvas(keys, title, _known());
          moved.fold(
            (error) => _fail(emit, error),
            (done) => _show(emit, done.$2, selection: const <StrokeKey>[]),
          );
        }
      case CanvasTextCardAddedEvent(:final x, :final y, :final text):
        await _change(emit, (doc) => doc.addTextCard(x, y, text, _known()));
      case CanvasNoteCardAddedEvent(:final noteId, :final x, :final y):
        await _change(emit, (doc) => doc.addNoteCard(noteId, x, y, _known()));
      case CanvasCardTextChangedEvent(:final id, :final text):
        await _change(emit, (doc) => doc.setCardText(id, text, _known()));
      case CanvasCardMovedEvent(:final id, :final rect):
        await _change(emit, (doc) => doc.setCardRect(id, rect, _known()));
      case CanvasCardDeletedEvent(:final id):
        await _change(emit, (doc) => doc.deleteCard(id, _known()));
      case CanvasArrowAddedEvent(:final from, :final to):
        await _change(emit, (doc) => doc.addArrow(from, to, _known()));
      case CanvasUndoneEvent():
        await _change(emit, (doc) => doc.undo(_known()));
      case CanvasRedoneEvent():
        await _change(emit, (doc) => doc.redo(_known()));
      case CanvasSaveRequestedEvent():
        await _save(emit);
      case CanvasMessageDismissedEvent():
        emit(state.copyWith(problem: null));
    }
  }

  /// Whether the document is an inline sketch in a note.
  bool _sketch = false;

  Future<void> _open(
    String noteId,
    Emitter<CanvasState> emit, {
    String? sketchId,
  }) async {
    emit(state.copyWith(status: CanvasStatus.loading));
    _sketch = sketchId != null;
    final opened = sketchId == null
        ? await _repository.open(noteId)
        : await _repository.openSketch(noteId, sketchId);
    final doc = opened.toOption().toNullable();
    if (doc == null) {
      final error = opened.swap().toOption().toNullable();
      emit(
        state.copyWith(
          status: error is LockedError
              ? CanvasStatus.locked
              : CanvasStatus.failed,
        ),
      );
      return;
    }
    _document = doc;
    final view = await doc.view(const <String>[]);
    view.fold(
      (_) => emit(state.copyWith(status: CanvasStatus.failed)),
      (view) => _show(emit, view, status: CanvasStatus.ready),
    );
  }

  /// Runs `call` on the document and shows what it returns. A stroke is
  /// written at once; anything else waits for the idle timer.
  Future<void> _change(
    Emitter<CanvasState> emit,
    Future<Either<ApplicationError, CanvasView>> Function(CanvasDocument doc)
    call, {
    bool saveNow = false,
    bool selectBorn = false,
    bool quietIfUnchanged = false,
  }) async {
    final doc = _document;
    if (doc == null) {
      return;
    }
    final shown = state.view;
    final result = await call(doc);
    // A whole erase gesture is one call; one that touched nothing changes
    // nothing, so it shows nothing and starts no save.
    final same = result.toOption().toNullable();
    if (quietIfUnchanged &&
        shown != null &&
        same != null &&
        _sameDocument(shown, same)) {
      return;
    }
    var failed = false;
    result.fold((error) {
      failed = true;
      _fail(emit, error);
    }, (view) => _show(emit, view, selectBorn: selectBorn));
    if (failed) {
      return;
    }
    if (saveNow) {
      _idle?.cancel();
      await _save(emit);
    } else {
      _idle?.cancel();
      _idle = Timer(canvasIdleSave, () {
        if (!isClosed) {
          add(const CanvasEvent.saveRequested());
        }
      });
    }
  }

  Future<void> _save(Emitter<CanvasState> emit) async {
    final doc = _document;
    if (doc == null || !(state.view?.dirty ?? false)) {
      return;
    }
    final saved = await doc.save();
    await saved.fold(
      (_) async => emit(state.copyWith(problem: CanvasProblem.notSaved)),
      (done) async {
        // A save that merged a change from elsewhere changes the view.
        if (done.merged) {
          final fresh = await doc.view(_known());
          fresh.fold((_) {}, (view) => _show(emit, view));
        }
        emit(
          state.copyWith(
            view: state.view?.copyWithDirty(dirty: false),
            problem: null,
          ),
        );
      },
    );
  }

  /// Whether `a` and `b` show the same document: the same strokes, the same
  /// history and the same size.
  bool _sameDocument(CanvasView a, CanvasView b) {
    if (a.bytes != b.bytes ||
        a.canUndo != b.canUndo ||
        a.canRedo != b.canRedo ||
        a.strokes.length != b.strokes.length) {
      return false;
    }
    for (var i = 0; i < a.strokes.length; i++) {
      if (a.strokes[i].id != b.strokes[i].id) {
        return false;
      }
    }
    return true;
  }

  /// The IDs of the strokes the screen holds outlines for.
  List<String> _known() => _held.keys.toList(growable: false);

  /// Shows `view`, keeping the outlines the screen already holds for the
  /// strokes it didn't ask to have drawn again.
  void _show(
    Emitter<CanvasState> emit,
    CanvasView view, {
    CanvasStatus? status,
    List<StrokeKey>? selection,
    bool selectBorn = false,
  }) {
    final strokes = <CanvasStroke>[];
    final present = <String>{};
    final born = <StrokeKey>[];
    for (final stroke in view.strokes) {
      present.add(stroke.id);
      if (stroke.known) {
        final held = _held[stroke.id];
        strokes.add(held ?? stroke);
      } else {
        if (!_held.containsKey(stroke.id)) {
          born.add(StrokeKey(nodeId: stroke.nodeId, strokeId: stroke.id));
        }
        _held[stroke.id] = stroke;
        strokes.add(stroke);
      }
    }
    _held.removeWhere((id, _) => !present.contains(id));
    emit(
      state.copyWith(
        status: status ?? state.status,
        view: view.withStrokes(strokes),
        edit: state.edit + 1,
        selection: selectBorn
            ? born
            : selection ?? _stillThere(state.selection, present),
        problem: null,
      ),
    );
  }

  List<StrokeKey> _stillThere(List<StrokeKey> keys, Set<String> present) =>
      keys.where((k) => present.contains(k.strokeId)).toList(growable: false);

  void _fail(Emitter<CanvasState> emit, ApplicationError error) {
    emit(
      state.copyWith(
        // A sketch that refuses ink says it is full (its cap is its own).
        problem:
            state.view?.budget == CanvasBudget.full ||
                (_sketch && error is InvalidError)
            ? CanvasProblem.full
            : CanvasProblem.failed,
      ),
    );
  }

  @override
  Future<void> close() async {
    _idle?.cancel();
    final doc = _document;
    if (doc != null && (state.view?.dirty ?? false)) {
      await doc.save();
    }
    return super.close();
  }
}

/// Small helpers on the bridge's view that it can't have itself.
extension CanvasViewCopy on CanvasView {
  CanvasView withStrokes(List<CanvasStroke> strokes) => CanvasView(
    noteId: noteId,
    rev: rev,
    title: title,
    strokes: strokes,
    cards: cards,
    groups: groups,
    arrows: arrows,
    bounds: bounds,
    bytes: bytes,
    budget: budget,
    canUndo: canUndo,
    canRedo: canRedo,
    dirty: dirty,
  );

  CanvasView copyWithDirty({required bool dirty}) => CanvasView(
    noteId: noteId,
    rev: rev,
    title: title,
    strokes: strokes,
    cards: cards,
    groups: groups,
    arrows: arrows,
    bounds: bounds,
    bytes: bytes,
    budget: budget,
    canUndo: canUndo,
    canRedo: canRedo,
    dirty: dirty,
  );
}
