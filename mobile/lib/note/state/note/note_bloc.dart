// Kanso — note_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/utils/debounce.dart';
import '../../../core/utils/source_lines.dart';
import '../../models/data/note_mode.dart';
import '../../models/data/note_place.dart';
import '../../models/data/note_screen_status.dart';
import '../../models/data/source_line.dart';
import '../../models/repos/abs/note_place_repository.dart';

part 'note_bloc.freezed.dart';
part 'note_event.dart';
part 'note_state.dart';

/// How long the writes of a burst may be apart for the note to be read
/// again once, after the last.
const Duration noteDebounce = Duration(milliseconds: 100);

/// One note: read from the library, its text highlighted by the Rust core
/// in a built-in theme and cut into lines, kept up to date as the library
/// changes.
///
/// A note that is locked and closed is its title and its lock: its text is
/// never highlighted, cut or kept, whatever the library handed over.
///
/// While the note is edited ([NoteMode.edit]) the editor has the text; the
/// library's writes (the editor's own saves) are not read again until the
/// note is read again, in Preview.
@injectable
class NoteBloc extends Bloc<NoteEvent, NoteState> {
  NoteBloc(this._library, this._renderer, this._places)
    : super(const NoteState()) {
    on<NoteStartedEvent>(_onStarted, transformer: restartable());
    on<NoteChangesReceivedEvent>(
      _onChangesReceived,
      transformer: debounceRestartable<NoteChangesReceivedEvent>(noteDebounce),
    );
    on<NoteModeChangedEvent>(_onModeChanged);
    on<NotePlaceKeptEvent>(_onPlaceKept);
    on<NoteSyntaxThemeChangedEvent>(
      _onSyntaxThemeChanged,
      transformer: restartable(),
    );
    on<NoteRetriedEvent>(
      (event, emit) => _read(emit),
      transformer: droppable(),
    );
  }

  final LibraryRepository _library;
  final RendererRepository _renderer;
  final NotePlaceRepository _places;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  /// Whether the note was written while it was edited, and so is out of
  /// date in the state.
  bool _stale = false;

  Future<void> _onStarted(
    NoteStartedEvent event,
    Emitter<NoteState> emit,
  ) async {
    // A note just made opens ready to write in.
    final fresh = _places.takeOpenedNew(event.id);
    emit(
      NoteState(
        noteId: event.id,
        themeId: event.themeId,
        mode: fresh ? NoteMode.edit : NoteMode.preview,
        openedNew: fresh,
      ),
    );
    _stale = false;
    await _changes?.cancel();
    _changes = _library.changes().listen((ids) {
      // Only a write to this note reads it again.
      if (ids.contains(event.id)) {
        add(NoteEvent.changesReceived(ids));
      }
    });
    await _read(emit);
  }

  Future<void> _onChangesReceived(
    NoteChangesReceivedEvent event,
    Emitter<NoteState> emit,
  ) async {
    if (state.mode == NoteMode.edit) {
      _stale = true;
      return;
    }
    await _read(emit);
  }

  Future<void> _onModeChanged(
    NoteModeChangedEvent event,
    Emitter<NoteState> emit,
  ) async {
    final editing = event.mode == NoteMode.edit;
    final tapped = event.bodyCaret;
    final place = editing ? _places.placeOf(state.noteId) : null;
    emit(
      state.copyWith(
        mode: event.mode,
        caret: !editing ? null : tapped ?? place?.caret,
        caretInBody: editing && tapped != null,
        // A tap names where to look; otherwise the page is where it was.
        scroll: tapped == null ? place?.scroll ?? 0 : 0,
      ),
    );
    if (!editing && _stale) {
      _stale = false;
      await _read(emit);
    }
  }

  void _onPlaceKept(NotePlaceKeptEvent event, Emitter<NoteState> emit) {
    if (state.noteId.isNotEmpty) {
      _places.keep(
        state.noteId,
        NotePlace(caret: event.caret, scroll: event.scroll),
      );
    }
  }

  Future<void> _onSyntaxThemeChanged(
    NoteSyntaxThemeChangedEvent event,
    Emitter<NoteState> emit,
  ) async {
    if (event.themeId == state.themeId) {
      return;
    }
    emit(state.copyWith(themeId: event.themeId));
    final body = state.note?.body;
    if (state.status != NoteScreenStatus.ready || body == null) {
      return;
    }
    final lines = await _lines(body, event.themeId);
    if (!emit.isDone && state.themeId == event.themeId) {
      emit(state.copyWith(lines: lines));
    }
  }

  /// Reads the note. A reading that a later one overtook is dropped, so the
  /// state never goes back to older data.
  Future<void> _read(Emitter<NoteState> emit) async {
    final reading = ++_reading;
    final result = await _library.note(state.noteId);
    if (reading != _reading || emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(
        error is NotFoundError
            ? state.copyWith(
                status: NoteScreenStatus.notFound,
                note: null,
                lines: const <SourceLine>[],
                error: null,
              )
            : state.copyWith(status: NoteScreenStatus.failed, error: error),
      ),
      (note) async {
        final body = note.body;
        if (note.lock is LockState_Closed || body == null) {
          emit(
            state.copyWith(
              status: NoteScreenStatus.locked,
              note: note,
              lines: const <SourceLine>[],
              error: null,
            ),
          );
          return;
        }
        final themeId = state.themeId;
        final lines = await _lines(body, themeId);
        if (reading != _reading || emit.isDone) {
          return;
        }
        emit(
          state.copyWith(
            status: NoteScreenStatus.ready,
            note: note,
            lines: lines,
            error: null,
          ),
        );
      },
    );
  }

  /// [body] in lines, with its runs in [themeId]. Text the highlighter
  /// cannot style is shown plain: a note is still there to read.
  Future<List<SourceLine>> _lines(String body, String themeId) async {
    final runs = await _renderer.styledRuns(
      body,
      start: 0,
      end: body.length,
      themeId: themeId,
    );
    return sourceLines(
      body,
      runs.getOrElse(() => const <StyledRun>[]),
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
