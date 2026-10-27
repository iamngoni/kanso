// Kanso — note_creation_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../note/models/repos/abs/note_place_repository.dart';
import '../../models/data/note_creation_failure.dart';
import '../../models/data/note_opening.dart';

part 'note_creation_bloc.freezed.dart';
part 'note_creation_event.dart';
part 'note_creation_state.dart';

/// Makes new notes: from the list's + button, from ⌘N, and from a link in a
/// note to a note that is not there yet.
///
/// The core decides which notebook a note goes in (see
/// `kanso_engine::list::new_note_target`): the notebook of the list it is
/// made from, else the Default notebook of the settings if that one exists
/// and is not locked, else the first. A note made from a tag's list starts
/// with the tag, one made from Pinned is pinned. Making a note while another
/// is being made is ignored, so a double tap makes one.
///
/// The note made is handed on as [NoteCreationState.opening] for the screen
/// to go to, and marked so that it opens ready to write in.
@injectable
class NoteCreationBloc extends Bloc<NoteCreationEvent, NoteCreationState> {
  NoteCreationBloc(this._library, this._places, this._preferences)
    : super(const NoteCreationState()) {
    on<NoteCreationRequestedEvent>(_onRequested, transformer: droppable());
    on<NoteCreationTitledEvent>(_onTitled, transformer: droppable());
    on<NoteCreationOpeningHandledEvent>(
      (event, emit) => emit(state.copyWith(opening: null)),
    );
    on<NoteCreationNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(failure: null)),
    );
  }

  final LibraryRepository _library;
  final NotePlaceRepository _places;
  final PreferencesRepository _preferences;

  Future<void> _onRequested(
    NoteCreationRequestedEvent event,
    Emitter<NoteCreationState> emit,
  ) async {
    await _create(
      NewNote(
        title: '',
        body: '',
        place: event.place,
        pinned: event.pinned,
        preferredNotebookId: await _preferredNotebook(),
      ),
      event.place,
      emit,
    );
  }

  Future<void> _onTitled(
    NoteCreationTitledEvent event,
    Emitter<NoteCreationState> emit,
  ) async {
    await _create(
      NewNote(
        title: event.title,
        body: '',
        preferredNotebookId: await _preferredNotebook(),
      ),
      const ListPlace.all(),
      emit,
    );
  }

  /// The notebook the person chose for new notes, if they did.
  Future<String?> _preferredNotebook() async {
    final loaded = await _preferences.load();
    return loaded.fold((_) => null, (found) => found.defaultNotebookId);
  }

  Future<void> _create(
    NewNote note,
    ListPlace from,
    Emitter<NoteCreationState> emit,
  ) async {
    emit(state.copyWith(creating: true, failure: null));
    final made = await _library.create(note);
    await made.fold(
      (error) async => emit(
        NoteCreationState(
          failure: error is LockedError
              ? NoteCreationFailure.locked
              : NoteCreationFailure.failed,
        ),
      ),
      (saved) async {
        _places.markOpenedNew(saved.id);
        var place = from;
        if (from is ListPlace_Trash) {
          // The trash cannot hold a new note: it went into a notebook, and
          // that is where it is opened.
          final read = await _library.note(saved.id);
          place = read.fold(
            (_) => const ListPlace.all(),
            (view) => ListPlace.notebook(id: view.notebookId),
          );
        }
        emit(
          NoteCreationState(
            opening: NoteOpening(noteId: saved.id, place: place),
          ),
        );
      },
    );
  }
}
