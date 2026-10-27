// Kanso — note_list_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/list_options.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/day_grouping.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/note_list_entry.dart';
import '../../models/data/note_list_status.dart';

part 'note_list_bloc.freezed.dart';
part 'note_list_event.dart';
part 'note_list_state.dart';

/// How long the writes of a burst may be apart for the list to be read
/// again once, after the last.
const Duration noteListDebounce = Duration(milliseconds: 100);

/// The notes of one place: read with the person's sort and narrowing, laid
/// out under date headers as the library says, kept up to date as the
/// library changes, and chosen and changed several at a time.
@injectable
class NoteListBloc extends Bloc<NoteListEvent, NoteListState> {
  NoteListBloc(
    this._library,
    this._preferences,
    this._haptics,
    this._undoNotice,
  ) : super(const NoteListState()) {
    on<NoteListStartedEvent>(_onStarted, transformer: restartable());
    on<NoteListOptionsChangedEvent>(
      _onOptionsChanged,
      transformer: restartable(),
    );
    on<NoteListChangesReceivedEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<NoteListChangesReceivedEvent>(
        noteListDebounce,
      ),
    );
    on<NoteListRetriedEvent>((event, emit) => _read(emit));
    on<NoteListSelectionStartedEvent>(_onSelectionStarted);
    on<NoteListSelectionToggledEvent>(_onSelectionToggled);
    on<NoteListSelectionClearedEvent>(
      (event, emit) => emit(state.copyWith(selected: const <String>{})),
    );
    on<NoteListChangeRequestedEvent>(
      _onChangeRequested,
      transformer: sequential(),
    );
    on<NoteListSwipeArmedEvent>((event, emit) => _haptics.impact());
    on<NoteListErrorDismissedEvent>(
      (event, emit) => emit(state.copyWith(actionError: null)),
    );
  }

  final LibraryRepository _library;
  final PreferencesRepository _preferences;
  final HapticsRepository _haptics;
  final UndoNoticeBloc _undoNotice;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  Future<void> _onStarted(
    NoteListStartedEvent event,
    Emitter<NoteListState> emit,
  ) async {
    emit(NoteListState(place: event.place));
    final saved = await _preferences.load();
    saved.fold(
      (_) {},
      (preferences) => emit(
        state.copyWith(
          options: ListOptions(
            sort: preferences.sort,
            descending: preferences.descending,
          ),
        ),
      ),
    );
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(NoteListEvent.changesReceived(ids)),
    );
    await _read(emit);
  }

  Future<void> _onOptionsChanged(
    NoteListOptionsChangedEvent event,
    Emitter<NoteListState> emit,
  ) async {
    final before = state.options;
    final after = event.options;
    emit(state.copyWith(options: after));
    if (before.sort != after.sort || before.descending != after.descending) {
      // The sort is one choice for every list; the narrowing is not kept.
      await _preferences.update(
        (current) =>
            current.copyWith(sort: after.sort, descending: after.descending),
      );
    }
    await _read(emit);
  }

  /// Reads the list. A reading that a later one overtook is dropped, so the
  /// state never goes back to older data.
  Future<void> _read(Emitter<NoteListState> emit) async {
    final reading = ++_reading;
    final place = state.place;
    final result = await _library.list(state.options.queryFor(place));
    if (reading != _reading || emit.isDone) {
      return;
    }
    result.fold(
      (error) =>
          emit(state.copyWith(status: NoteListStatus.failed, error: error)),
      (list) {
        final present = <String>{for (final row in list.rows) row.id};
        emit(
          state.copyWith(
            status: NoteListStatus.ready,
            rows: list.rows,
            shape: list.shape,
            entries: groupRows(list.rows, list.shape),
            selected: state.selected.intersection(present),
            error: null,
          ),
        );
      },
    );
  }

  Future<void> _onSelectionStarted(
    NoteListSelectionStartedEvent event,
    Emitter<NoteListState> emit,
  ) async {
    emit(state.copyWith(selected: <String>{event.id}));
    await _haptics.selection();
  }

  Future<void> _onSelectionToggled(
    NoteListSelectionToggledEvent event,
    Emitter<NoteListState> emit,
  ) async {
    final selected = <String>{...state.selected};
    if (!selected.remove(event.id)) {
      selected.add(event.id);
    }
    emit(state.copyWith(selected: selected));
    await _haptics.selection();
  }

  Future<void> _onChangeRequested(
    NoteListChangeRequestedEvent event,
    Emitter<NoteListState> emit,
  ) async {
    final result = await _library.change(event.ids, event.change);
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(state.copyWith(actionError: error)),
      (changed) async {
        final ticket = changed.undo;
        if (ticket != null) {
          _undoNotice.add(UndoNoticeEvent.shown(ticket, changed.message));
        }
        // What was chosen has been dealt with.
        emit(state.copyWith(selected: const <String>{}, actionError: null));
        if (event.change is NoteChange_TogglePin) {
          await _haptics.impact();
        }
        await _read(emit);
      },
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
