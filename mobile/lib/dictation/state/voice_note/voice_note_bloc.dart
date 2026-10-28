// Kanso — voice_note_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../models/data/voice_note_status.dart';
import '../../utils/split_voice_note.dart';

part 'voice_note_bloc.freezed.dart';
part 'voice_note_event.dart';
part 'voice_note_state.dart';

/// Saves what was dictated as a new note, titled by its first line as the
/// desktop's quick capture does.
@injectable
class VoiceNoteBloc extends Bloc<VoiceNoteEvent, VoiceNoteState> {
  VoiceNoteBloc(this.library) : super(const VoiceNoteState()) {
    on<VoiceNoteSaveRequestedEvent>(onSaveRequested, transformer: droppable());
  }

  final LibraryRepository library;

  Future<void> onSaveRequested(
    VoiceNoteSaveRequestedEvent event,
    Emitter<VoiceNoteState> emit,
  ) async {
    final text = splitVoiceNote(event.transcript);
    if (text == null) {
      return;
    }
    emit(const VoiceNoteState(status: VoiceNoteStatus.saving));
    final saved = await library.create(
      NewNote(title: text.title, body: text.body),
    );
    emit(
      saved.fold(
        (error) => VoiceNoteState(status: VoiceNoteStatus.failed, error: error),
        (made) =>
            VoiceNoteState(status: VoiceNoteStatus.saved, noteId: made.id),
      ),
    );
  }
}
