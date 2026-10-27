// Kanso — editor_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart' show AppLifecycleState;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../capture/models/data/media_source.dart';
import '../../../capture/models/repos/abs/media_picker_repository.dart';
import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/bridge/api/types.dart' as core;
import '../../../core/models/data/application_error.dart';
import '../../models/data/editor_destination.dart';
import '../../models/data/editor_status.dart';
import '../../models/data/flush_reason.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../models/repos/abs/web_link_repository.dart';

part 'editor_bloc.freezed.dart';
part 'editor_event.dart';
part 'editor_state.dart';

/// What the open note's session says that it was not asked: a save, a change
/// made elsewhere, a reload, a failure, its end.
typedef EditorSignal = core.EditorEvent;

/// A note open for editing: opened, kept saved, asked about a change made
/// elsewhere, and let go.
///
/// The text itself is in the field and the Rust core, one keystroke at a
/// time (see `KansoMarkdownController`); this holds the rest: whether the
/// note is saved, what completes under the caret, the properties, and the
/// answers to the conflict and the lock.
///
/// After the note's lock closes or the note is gone, nothing of its text is
/// kept: the state holds none.
@injectable
class EditorBloc extends Bloc<EditorEvent, EditorState> {
  EditorBloc(this._editors, this._links, this._picker)
    : super(const EditorState()) {
    on<EditorStartedEvent>(_onStarted, transformer: restartable());
    on<EditorSignalReceivedEvent>(_onSignal, transformer: sequential());
    on<EditorEditedEvent>(_onEdited);
    on<EditorCaretMovedEvent>(_onCaretMoved, transformer: restartable());
    on<EditorCompletionDismissedEvent>(_onCompletionDismissed);
    on<EditorPropertiesToggledEvent>(_onPropertiesToggled);
    on<EditorPropertySetEvent>(_onPropertySet, transformer: sequential());
    on<EditorPropertyRemovedEvent>(
      _onPropertyRemoved,
      transformer: sequential(),
    );
    on<EditorKeepMineRequestedEvent>(_onKeepMine, transformer: droppable());
    on<EditorUseTheirsRequestedEvent>(_onUseTheirs, transformer: droppable());
    on<EditorFlushRequestedEvent>(_onFlush, transformer: sequential());
    on<EditorLifecycleChangedEvent>(_onLifecycle, transformer: sequential());
    on<EditorLinkOpenRequestedEvent>(_onLinkOpen, transformer: droppable());
    on<EditorDestinationHandledEvent>(_onDestinationHandled);
    on<EditorClosedEvent>(_onClosed, transformer: sequential());
    on<EditorAttachRequestedEvent>(_onAttach, transformer: droppable());
    on<EditorAttachNoticeDismissedEvent>(
      (event, emit) => emit(
        state.copyWith(attachProblems: const <ApplicationError>[]),
      ),
    );
  }

  final EditorRepository _editors;
  final WebLinkRepository _links;
  final MediaPickerRepository _picker;
  EditorDocument? _document;
  StreamSubscription<EditorSignal>? _events;

  /// Picks from the source, and attaches each file picked in turn where
  /// the caret is then; why any was refused is kept to say.
  Future<void> _onAttach(
    EditorAttachRequestedEvent event,
    Emitter<EditorState> emit,
  ) async {
    final picked = await _picker.pick(event.source);
    final problems = <ApplicationError>[];
    final files = picked.fold((error) {
      problems.add(error);
      return const <ShareFile>[];
    }, (files) => files);
    for (final file in files) {
      final document = _document;
      if (document == null || isClosed) {
        return;
      }
      final attached = await document.attach(file);
      attached.fold(
        problems.add,
        (outcome) => emit(
          state.copyWith(attached: outcome, attachSeq: state.attachSeq + 1),
        ),
      );
    }
    if (problems.isNotEmpty) {
      emit(state.copyWith(attachProblems: problems));
    }
  }

  Future<void> _onStarted(
    EditorStartedEvent event,
    Emitter<EditorState> emit,
  ) async {
    await _release();
    emit(EditorState(noteId: event.noteId));
    final opened = await _editors.open(event.noteId);
    final document = opened.fold<EditorDocument?>((error) {
      emit(_unavailable(event.noteId, error));
      return null;
    }, (document) => document);
    if (document == null) {
      return;
    }
    final read = await document.snapshot();
    if (emit.isDone) {
      unawaited(document.close());
      return;
    }
    read.fold(
      (error) {
        unawaited(document.close());
        emit(_unavailable(event.noteId, error));
      },
      (snapshot) {
        _document = document;
        _events = document.events().listen(
          (signal) => add(EditorEvent.signalReceived(signal)),
          onError: (Object _) {},
        );
        emit(
          EditorState(
            noteId: event.noteId,
            status: EditorStatus.ready,
            document: document,
            snapshot: snapshot,
            save: snapshot.save,
            properties: snapshot.properties,
          ),
        );
      },
    );
  }

  EditorState _unavailable(String noteId, ApplicationError error) {
    return EditorState(
      noteId: noteId,
      status: switch (error) {
        LockedError() => EditorStatus.locked,
        NotFoundError() => EditorStatus.notFound,
        _ => EditorStatus.failed,
      },
      error: error is LockedError || error is NotFoundError ? null : error,
    );
  }

  Future<void> _onSignal(
    EditorSignalReceivedEvent event,
    Emitter<EditorState> emit,
  ) async {
    if (state.status != EditorStatus.ready) {
      return;
    }
    switch (event.signal) {
      case core.EditorEvent_Saved(:final rev, :final atMs):
        emit(
          state.copyWith(
            save: SaveState.saved(rev: rev, atMs: atMs),
          ),
        );
      case core.EditorEvent_Incoming(:final deleted):
        emit(state.copyWith(save: SaveState.conflict(deleted: deleted)));
      case core.EditorEvent_Reloaded(:final snapshot):
        emit(
          state.copyWith(
            snapshot: snapshot,
            save: snapshot.save,
            properties: snapshot.properties,
            completion: null,
            reloadSeq: state.reloadSeq + 1,
          ),
        );
      case core.EditorEvent_Failed(:final message):
        emit(state.copyWith(save: SaveState.failed(message: message)));
      case core.EditorEvent_Closed():
        await _release();
        emit(EditorState(noteId: state.noteId, status: EditorStatus.locked));
    }
  }

  void _onEdited(EditorEditedEvent event, Emitter<EditorState> emit) {
    if (state.status == EditorStatus.ready &&
        state.save is! SaveState_Conflict) {
      emit(state.copyWith(save: const SaveState.pending()));
    }
  }

  Future<void> _onCaretMoved(
    EditorCaretMovedEvent event,
    Emitter<EditorState> emit,
  ) async {
    final document = _document;
    if (state.status != EditorStatus.ready || document == null) {
      return;
    }
    emit(state.copyWith(context: event.context));
    if (!event.context.completionHere) {
      if (state.completion != null) {
        emit(state.copyWith(completion: null));
      }
      return;
    }
    final result = await document.completion();
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(completion: null)),
      (completion) => emit(state.copyWith(completion: completion)),
    );
  }

  void _onCompletionDismissed(
    EditorCompletionDismissedEvent event,
    Emitter<EditorState> emit,
  ) {
    if (state.completion != null) {
      emit(state.copyWith(completion: null));
    }
  }

  void _onPropertiesToggled(
    EditorPropertiesToggledEvent event,
    Emitter<EditorState> emit,
  ) {
    emit(state.copyWith(propertiesOpen: !state.propertiesOpen));
  }

  Future<void> _onPropertySet(
    EditorPropertySetEvent event,
    Emitter<EditorState> emit,
  ) async {
    final document = _document;
    if (document == null) {
      return;
    }
    final result = await document.setProperty(
      index: event.index,
      key: event.key,
      value: event.value,
    );
    _adoptProperties(result, emit);
  }

  Future<void> _onPropertyRemoved(
    EditorPropertyRemovedEvent event,
    Emitter<EditorState> emit,
  ) async {
    final document = _document;
    if (document == null) {
      return;
    }
    _adoptProperties(await document.removeProperty(event.index), emit);
  }

  void _adoptProperties(
    Either<ApplicationError, List<Property>> result,
    Emitter<EditorState> emit,
  ) {
    result.fold(
      (error) => emit(state.copyWith(propertyError: error)),
      (properties) => emit(
        state.copyWith(
          properties: properties,
          propertyError: null,
          save: state.save is SaveState_Conflict
              ? state.save
              : const SaveState.pending(),
        ),
      ),
    );
  }

  Future<void> _onKeepMine(
    EditorKeepMineRequestedEvent event,
    Emitter<EditorState> emit,
  ) async {
    final document = _document;
    if (document == null) {
      return;
    }
    final result = await document.keepMine();
    result.fold(
      (error) => emit(state.copyWith(save: _failure(error))),
      (save) => emit(state.copyWith(save: save)),
    );
  }

  Future<void> _onUseTheirs(
    EditorUseTheirsRequestedEvent event,
    Emitter<EditorState> emit,
  ) async {
    final document = _document;
    if (document == null) {
      return;
    }
    final result = await document.useTheirs();
    await result.fold(
      (error) async {
        if (error is NotFoundError) {
          // Deleted elsewhere: the session is over.
          await _release();
          emit(
            EditorState(noteId: state.noteId, status: EditorStatus.notFound),
          );
        } else {
          emit(state.copyWith(save: _failure(error)));
        }
      },
      (snapshot) async {
        emit(
          state.copyWith(
            snapshot: snapshot,
            save: snapshot.save,
            properties: snapshot.properties,
            completion: null,
            reloadSeq: state.reloadSeq + 1,
          ),
        );
      },
    );
  }

  Future<void> _onFlush(
    EditorFlushRequestedEvent event,
    Emitter<EditorState> emit,
  ) => _flush(event.reason, emit);

  Future<void> _onLifecycle(
    EditorLifecycleChangedEvent event,
    Emitter<EditorState> emit,
  ) async {
    switch (event.lifecycle) {
      case AppLifecycleState.inactive:
      case AppLifecycleState.hidden:
      case AppLifecycleState.paused:
        await _flush(FlushReason.lifecycle, emit);
      case AppLifecycleState.resumed:
      case AppLifecycleState.detached:
        break;
    }
  }

  /// Saves now. With a change made elsewhere waiting for an answer, only
  /// leaving the editor answers it (by keeping what was typed): going to
  /// the background or pressing save leaves the question open.
  Future<void> _flush(FlushReason reason, Emitter<EditorState> emit) async {
    final document = _document;
    if (document == null || state.status != EditorStatus.ready) {
      return;
    }
    final leaving =
        reason == FlushReason.preview || reason == FlushReason.navigation;
    if (state.save is SaveState_Conflict && !leaving) {
      return;
    }
    final result = await document.flush();
    if (emit.isDone) {
      return;
    }
    result.fold((error) {
      if (error is LockedError) {
        emit(EditorState(noteId: state.noteId, status: EditorStatus.locked));
      } else {
        emit(state.copyWith(save: _failure(error)));
      }
    }, (save) => emit(state.copyWith(save: save)));
  }

  /// Opens what a link in the text leads to. A note, or a note made for a
  /// link to one that is not there yet, is gone to once what was typed is
  /// saved; an address goes to the app that opens it.
  Future<void> _onLinkOpen(
    EditorLinkOpenRequestedEvent event,
    Emitter<EditorState> emit,
  ) async {
    switch (event.target) {
      case LinkTarget_Note(:final id):
        await _flush(FlushReason.navigation, emit);
        emit(state.copyWith(destination: EditorDestination.note(id)));
      case LinkTarget_NewNote(:final title):
        // The note is made by the app's `NoteCreationBloc`, as any new note
        // is, once what was typed is saved.
        await _flush(FlushReason.navigation, emit);
        emit(
          state.copyWith(
            destination: EditorDestination.newNote(title),
            linkError: null,
          ),
        );
      case LinkTarget_Web(:final url):
        final opened = await _links.open(url);
        opened.fold(
          (error) => emit(state.copyWith(linkError: error)),
          (_) => emit(state.copyWith(linkError: null)),
        );
    }
  }

  void _onDestinationHandled(
    EditorDestinationHandledEvent event,
    Emitter<EditorState> emit,
  ) {
    emit(state.copyWith(destination: null));
  }

  Future<void> _onClosed(
    EditorClosedEvent event,
    Emitter<EditorState> emit,
  ) async {
    await _release();
    emit(EditorState(noteId: state.noteId));
  }

  SaveState _failure(ApplicationError error) {
    return SaveState.failed(message: error.message);
  }

  /// Saves and lets go of the open note.
  Future<void> _release() async {
    final document = _document;
    _document = null;
    await _events?.cancel();
    _events = null;
    if (document != null) {
      // Leaving saves what was typed, keeping it over a change made
      // elsewhere. A note whose lock closed has been saved already.
      await document.close();
    }
  }

  @override
  Future<void> close() async {
    await _release();
    return super.close();
  }
}
