// Kanso — preview_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart' show Either, Left;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/models/repos/abs/themes_repository.dart';
import '../../../core/services/preview_theme_fitter.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../editor/models/repos/abs/web_link_repository.dart';
import '../../models/data/preview_action.dart';

export '../../../core/models/data/preview_themes.dart';

part 'preview_bloc.freezed.dart';
part 'preview_event.dart';
part 'preview_state.dart';

/// The address a due chip's "Stop repeating" taps, with the task's place in
/// the note's task list after it (`kanso-stop-repeat:2`).
const String stopRepeatingLink = 'kanso-stop-repeat:';

/// The body size of the page, in points: the desktop preview's.
const double previewBaseTextSize = 18;

/// The rendered page size retained by exports that do not receive
/// appearance preferences.
const double previewTextSize = previewBaseTextSize;

/// A note's preview, as the blocks the app draws, and what a tap in it does.
///
/// The blocks are the core's (`previewBlocks`): it decides what every link
/// is and keeps the rest out. A tap is handed back to the core
/// (`linkTarget`), and only its answer is acted on: a box is ticked here, in
/// one write with the Undo notice; a note to make is made here; the rest
/// goes to the screen as a [PreviewAction]. The blocks are drawn again when
/// anything in the library changes, since a query view or a link can show
/// any note, and when the themes change, since code and drawings are
/// coloured by them.
@injectable
class PreviewBloc extends Bloc<PreviewEvent, PreviewState> {
  PreviewBloc(
    this._library,
    this._renderer,
    ThemesRepository themes,
    this._links,
    this._undoNotice,
    this._haptics,
  ) : _fitter = PreviewThemeFitter(themes),
      super(const PreviewState()) {
    on<PreviewStartedEvent>(_onStarted, transformer: restartable());
    on<PreviewThemeChangedEvent>(_onThemeChanged, transformer: restartable());
    on<PreviewStyleChangedEvent>(_onStyleChanged, transformer: restartable());
    on<PreviewChangesReceivedEvent>(_onChanges, transformer: droppable());
    on<PreviewLinkTappedEvent>(_onLinkTapped, transformer: sequential());
    on<PreviewActionHandledEvent>(
      (event, emit) => emit(state.copyWith(action: null)),
    );
    on<PreviewNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(notice: null)),
    );
  }

  final LibraryRepository _library;
  final RendererRepository _renderer;
  final WebLinkRepository _links;
  final UndoNoticeBloc _undoNotice;
  final HapticsRepository _haptics;

  /// Fits the themes the note is drawn in, as its exports are, and keeps the
  /// last fit so the same themes aren't fitted again.
  final PreviewThemeFitter _fitter;

  /// The number of the last draw begun: a draw only shows its blocks while
  /// no later one has begun, so an older draw that ends late can't replace
  /// a newer one.
  int _draws = 0;

  /// The themes and size of the blocks shown, so that a change back to them
  /// while a draw for others was cut short still draws again.
  (PreviewThemes, double)? _drawnWith;
  StreamSubscription<List<String>>? _changes;

  Future<void> _onStarted(
    PreviewStartedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    emit(
      PreviewState(
        noteId: event.noteId,
        themes: event.themes,
        size: event.size,
      ),
    );
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (_) => add(const PreviewEvent.changesReceived()),
    );
    await _draw(emit);
  }

  Future<void> _onThemeChanged(
    PreviewThemeChangedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    emit(state.copyWith(themes: event.themes));
    if (_drawnWith == (state.themes, state.size)) {
      return;
    }
    await _draw(emit);
  }

  Future<void> _onStyleChanged(
    PreviewStyleChangedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    emit(state.copyWith(size: event.size));
    if (_drawnWith == (state.themes, state.size)) {
      return;
    }
    await _draw(emit);
  }

  Future<void> _onChanges(
    PreviewChangesReceivedEvent event,
    Emitter<PreviewState> emit,
  ) => _draw(emit);

  /// Draws the note's blocks with the fitted preview palette and code
  /// colours, and keeps the revision they were drawn from. Only the latest
  /// draw begun shows what it drew.
  Future<void> _draw(Emitter<PreviewState> emit) async {
    final draw = ++_draws;
    final noteId = state.noteId;
    final themes = state.themes;
    final size = state.size;
    final tokens = await _fitter.tokens(themes);
    final note = await _library.note(noteId);
    if (emit.isDone || draw != _draws) {
      return;
    }
    final drawn = await tokens.fold(
      (error) async => Left<ApplicationError, (List<PreviewBlock>, String)>(
        error,
      ),
      (tokens) => note.fold(
        (error) async => Left<ApplicationError, (List<PreviewBlock>, String)>(
          error,
        ),
        (note) async {
          final blocks = await _renderer.previewBlocks(
            noteId,
            tokens.palette,
            PreviewOptions(
              size: size,
              utcOffsetSecs: clock.now().timeZoneOffset.inSeconds,
              code: tokens.syntax,
            ),
          );
          return blocks.map((blocks) => (blocks, note.rev));
        },
      ),
    );
    if (emit.isDone || draw != _draws || noteId != state.noteId) {
      return;
    }
    if (drawn.isRight()) {
      _drawnWith = (themes, size);
    }
    emit(
      drawn.fold(
        (error) => state.copyWith(error: error),
        (drawn) => state.copyWith(blocks: drawn.$1, rev: drawn.$2, error: null),
      ),
    );
  }

  Future<void> _onLinkTapped(
    PreviewLinkTappedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    final noteId = state.noteId;
    // "Stop repeating" from a due chip's menu names its task by place.
    if (event.url.startsWith(stopRepeatingLink)) {
      final index = int.tryParse(
        event.url.substring(stopRepeatingLink.length),
      );
      if (index != null && index >= 0) {
        await _write(
          () => _library.stopRepeatingAt(
            noteId: noteId,
            index: index,
            baseRev: state.rev,
          ),
          emit,
        );
      }
      return;
    }
    final target = await _renderer.linkTarget(noteId, event.url);
    if (emit.isDone) {
      return;
    }
    await target.fold(
      (error) async => emit(state.copyWith(notice: error)),
      (target) => _follow(target, emit),
    );
  }

  Future<void> _follow(PreviewLink target, Emitter<PreviewState> emit) async {
    final noteId = state.noteId;
    switch (target) {
      case PreviewLink_Note(:final id):
        emit(state.copyWith(action: PreviewAction.openNote(id)));
      case PreviewLink_NewNote(:final title):
        final made = await _library.createLinkedNote(
          title: title,
          fromNote: noteId,
        );
        made.fold(
          (error) => emit(state.copyWith(notice: error)),
          (saved) =>
              emit(state.copyWith(action: PreviewAction.openNote(saved.id))),
        );
      case PreviewLink_Tag(:final name):
        emit(state.copyWith(action: PreviewAction.openTag(name)));
      case PreviewLink_Heading(:final id):
        emit(state.copyWith(action: PreviewAction.scrollTo(id)));
      case PreviewLink_Web(:final url):
        final opened = await _links.open(url);
        opened.fold((error) => emit(state.copyWith(notice: error)), (_) {});
      case PreviewLink_Pdf(
        :final id,
        :final name,
        :final page,
        :final highlight,
      ):
        emit(
          state.copyWith(
            action: PreviewAction.openPdf(
              noteId: noteId,
              file: id,
              name: name,
              page: page,
              highlight: highlight,
            ),
          ),
        );
      case PreviewLink_Attachment(:final id):
        final file = await _library.attachment(id);
        file.fold(
          (error) => emit(state.copyWith(notice: error)),
          (file) =>
              emit(state.copyWith(action: PreviewAction.showAttachment(file))),
        );
      case PreviewLink_Task(:final index):
        await _tick(index, emit);
      case PreviewLink_CopyCode(:final index):
        final code = await _renderer.codeBlock(noteId, index);
        await code.fold((error) async => emit(state.copyWith(notice: error)), (
          code,
        ) async {
          emit(state.copyWith(action: PreviewAction.copyCode(index, code)));
          await _haptics.selection();
        });
      case PreviewLink_Audio(:final fileId, :final seconds):
        emit(state.copyWith(action: PreviewAction.playMoment(fileId, seconds)));
      case PreviewLink_Due() || PreviewLink_Nothing():
        break;
    }
  }

  /// Ticks the [index]-th box against the blocks shown: one write, with the
  /// Undo notice. Blocks older than the note are drawn again instead.
  Future<void> _tick(int index, Emitter<PreviewState> emit) => _write(
    () => _library.toggleTaskAt(
      noteId: state.noteId,
      index: index,
      baseRev: state.rev,
    ),
    emit,
  );

  /// A task write from the page ([write]): its Undo notice when it changed
  /// the note, and the blocks drawn again. Blocks older than the note are
  /// drawn again instead.
  Future<void> _write(
    Future<Either<ApplicationError, Ticked>> Function() write,
    Emitter<PreviewState> emit,
  ) async {
    final result = await write();
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async {
        if (error is ConflictError) {
          await _draw(emit);
        } else {
          emit(state.copyWith(notice: error));
        }
      },
      (ticked) async {
        switch (ticked) {
          case Ticked_Changed(:final rev, :final undo, :final message):
            emit(state.copyWith(rev: rev));
            _undoNotice.add(UndoNoticeEvent.shown(undo, message));
            await _haptics.selection();
          case Ticked_Unchanged() || Ticked_Missing():
            break;
        }
        await _draw(emit);
      },
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
