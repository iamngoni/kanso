// Kanso — preview_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart' show Left;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../editor/models/repos/abs/web_link_repository.dart';
import '../../models/data/preview_action.dart';

part 'preview_bloc.freezed.dart';
part 'preview_event.dart';
part 'preview_state.dart';

/// The body size of the page, in points: the desktop preview's.
const double previewTextSize = 16;

/// A note's preview page, and what a tap in it does.
///
/// The page is the core's (`previewPage`): it decides what every link is
/// and keeps the rest out. A tap is handed back to the core (`linkTarget`),
/// and only its answer is acted on: a box is ticked here, in one write with
/// the Undo notice; a note to make is made here; the rest goes to the
/// screen as a [PreviewAction]. The page is drawn again when anything in
/// the library changes, since a query view or a link can show any note.
@injectable
class PreviewBloc extends Bloc<PreviewEvent, PreviewState> {
  PreviewBloc(
    this._library,
    this._renderer,
    this._links,
    this._undoNotice,
    this._haptics,
  ) : super(const PreviewState()) {
    on<PreviewStartedEvent>(_onStarted, transformer: restartable());
    on<PreviewThemeChangedEvent>(_onThemeChanged, transformer: restartable());
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
  StreamSubscription<List<String>>? _changes;

  Future<void> _onStarted(
    PreviewStartedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    emit(PreviewState(noteId: event.noteId, themeId: event.themeId));
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
    if (event.themeId == state.themeId) {
      return;
    }
    emit(state.copyWith(themeId: event.themeId));
    await _draw(emit);
  }

  Future<void> _onChanges(
    PreviewChangesReceivedEvent event,
    Emitter<PreviewState> emit,
  ) => _draw(emit);

  /// Draws the page in the theme's palette, with its code colours, and
  /// keeps the revision it was drawn from.
  Future<void> _draw(Emitter<PreviewState> emit) async {
    final noteId = state.noteId;
    final tokens = await _renderer.builtinThemeTokens(state.themeId);
    final note = await _library.note(noteId);
    if (emit.isDone) {
      return;
    }
    final drawn = await tokens.fold(
      (error) async => Left<ApplicationError, (String, String)>(error),
      (tokens) => note.fold(
        (error) async => Left<ApplicationError, (String, String)>(error),
        (note) async {
          final page = await _renderer.previewPage(
            noteId,
            tokens.palette,
            PreviewOptions(
              size: previewTextSize,
              utcOffsetSecs: clock.now().timeZoneOffset.inSeconds,
              code: tokens.syntax,
            ),
          );
          return page.map((html) => (html, note.rev));
        },
      ),
    );
    if (emit.isDone || noteId != state.noteId) {
      return;
    }
    emit(
      drawn.fold(
        (error) => state.copyWith(error: error),
        (page) => state.copyWith(html: page.$1, rev: page.$2, error: null),
      ),
    );
  }

  Future<void> _onLinkTapped(
    PreviewLinkTappedEvent event,
    Emitter<PreviewState> emit,
  ) async {
    final noteId = state.noteId;
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
      case PreviewLink_Due() || PreviewLink_Nothing():
        break;
    }
  }

  /// Ticks the [index]-th box against the page shown: one write, with the
  /// Undo notice. A page older than the note is drawn again instead.
  Future<void> _tick(int index, Emitter<PreviewState> emit) async {
    final result = await _library.toggleTaskAt(
      noteId: state.noteId,
      index: index,
      baseRev: state.rev,
    );
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
