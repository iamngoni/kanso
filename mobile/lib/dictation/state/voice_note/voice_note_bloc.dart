// Kanso — voice_note_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/voice_capture_types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../voice_capture/models/repos/abs/voice_capture_repository.dart';
import '../../models/data/voice_note_status.dart';

part 'voice_note_bloc.freezed.dart';
part 'voice_note_event.dart';
part 'voice_note_state.dart';

/// Saves what was dictated as a new note in the Inbox (issue #193), titled
/// by its first sentence and recorded as a voice capture (`kansoCapture`),
/// as the desktop's quick capture and the Action Button do.
@injectable
class VoiceNoteBloc extends Bloc<VoiceNoteEvent, VoiceNoteState> {
  VoiceNoteBloc(this.voice) : super(const VoiceNoteState()) {
    // One queue: the clock starting and the save can't pass each other.
    on<VoiceNoteListeningStartedEvent>(
      onListeningStarted,
      transformer: sequential(),
    );
    on<VoiceNoteSaveRequestedEvent>(onSaveRequested, transformer: droppable());
  }

  final VoiceCaptureRepository voice;

  /// The time now; tests set their own.
  @visibleForTesting
  DateTime Function() now = DateTime.now;

  DateTime? _startedAt;

  void onListeningStarted(
    VoiceNoteListeningStartedEvent event,
    Emitter<VoiceNoteState> emit,
  ) {
    _startedAt ??= now();
  }

  Future<void> onSaveRequested(
    VoiceNoteSaveRequestedEvent event,
    Emitter<VoiceNoteState> emit,
  ) async {
    final transcript = event.transcript.trim();
    if (transcript.isEmpty) {
      return;
    }
    emit(const VoiceNoteState(status: VoiceNoteStatus.saving));
    final ended = now();
    final started = _startedAt ?? ended;
    _startedAt = null;
    final saved = await voice.save(
      VoiceCaptureInput(
        id: '',
        startedMs: started.millisecondsSinceEpoch,
        durationMs: ended.difference(started).inMilliseconds.clamp(0, 1 << 40),
        transcript: transcript,
        onDevice: true,
      ),
    );
    emit(
      saved.fold(
        (error) => VoiceNoteState(status: VoiceNoteStatus.failed, error: error),
        (made) {
          final id = made.noteId;
          if (id == null) {
            // Nothing was heard: nothing was saved.
            return const VoiceNoteState();
          }
          // Settings say Always and AI is on: tidy it in the background.
          if (made.tidyNow) {
            unawaited(voice.tidy(id));
          }
          return VoiceNoteState(status: VoiceNoteStatus.saved, noteId: id);
        },
      ),
    );
  }
}
