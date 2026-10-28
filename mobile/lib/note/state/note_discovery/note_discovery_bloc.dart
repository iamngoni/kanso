// Kanso — note_discovery_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';

part 'note_discovery_bloc.freezed.dart';
part 'note_discovery_event.dart';
part 'note_discovery_state.dart';

/// Related notes and meaning suggestions for one unlocked note.
@injectable
class NoteDiscoveryBloc extends Bloc<NoteDiscoveryEvent, NoteDiscoveryState> {
  NoteDiscoveryBloc(this._discovery) : super(const NoteDiscoveryState()) {
    on<NoteDiscoveryStartedEvent>(_onStarted);
    on<NoteDiscoveryTagAddedEvent>(_onTagAdded);
    on<NoteDiscoveryTwinDismissedEvent>(_onTwinDismissed);
    on<NoteDiscoveryNotesMergedEvent>(_onMerged);
  }

  final DiscoveryRepository _discovery;

  Future<void> _onStarted(
    NoteDiscoveryStartedEvent event,
    Emitter<NoteDiscoveryState> emit,
  ) async {
    emit(state.copyWith(noteId: event.noteId, loading: true));
    final enabled = await _discovery.relatedEnabled();
    final results = await (
      _discovery.relatedNotes(event.noteId, limit: 5),
      _discovery.noteMeaning(event.noteId),
    ).wait;
    emit(
      state.copyWith(
        loading: false,
        relatedEnabled: enabled.fold((_) => false, (value) => value),
        related: results.$1.fold((_) => const <RelatedRow>[], (rows) => rows),
        meaning: results.$2.fold((_) => null, (meaning) => meaning),
      ),
    );
  }

  Future<void> _onTagAdded(
    NoteDiscoveryTagAddedEvent event,
    Emitter<NoteDiscoveryState> emit,
  ) async {
    final result = await _discovery.addTag(state.noteId, event.tagId);
    if (result.isRight()) {
      await _onStarted(NoteDiscoveryStartedEvent(state.noteId), emit);
    }
  }

  Future<void> _onTwinDismissed(
    NoteDiscoveryTwinDismissedEvent event,
    Emitter<NoteDiscoveryState> emit,
  ) async {
    final result = await _discovery.dismissTwin(state.noteId, event.otherId);
    if (result.isRight()) {
      await _onStarted(NoteDiscoveryStartedEvent(state.noteId), emit);
    }
  }

  Future<void> _onMerged(
    NoteDiscoveryNotesMergedEvent event,
    Emitter<NoteDiscoveryState> emit,
  ) async {
    final result = await _discovery.mergeNotes(state.noteId, event.otherId);
    if (result.isRight()) {
      await _onStarted(NoteDiscoveryStartedEvent(state.noteId), emit);
    }
  }
}
