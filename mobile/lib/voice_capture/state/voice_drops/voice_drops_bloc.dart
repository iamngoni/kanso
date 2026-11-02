// Kanso — voice_drops_bloc.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/voice_capture_types.dart';
import '../../../dictation/models/data/file_transcript.dart';
import '../../../dictation/models/repos/abs/dictation_repository.dart';
import '../../../voice_settings/models/data/voice_tidy_mode.dart';
import '../../../voice_settings/models/repos/abs/voice_settings_repository.dart';
import '../../models/repos/abs/voice_capture_repository.dart';

part 'voice_drops_bloc.freezed.dart';
part 'voice_drops_event.dart';
part 'voice_drops_state.dart';

/// Turns the recordings made outside the app (the Action Button, Control
/// Center, the lock screen widget, Siri, the Android tile) into notes.
///
/// A recording is audio in a folder of the library's inbox. The intent that
/// made it never reads the library or a key, so the transcribing happens
/// here, when the app starts and each time it comes back to the front: the
/// phone's recognizer reads the file, and the words become a note in the
/// Inbox (`kansoCapture`, audio kept as Settings says). Then, if Settings
/// say Always and AI is on, the note is tidied.
///
/// Two passes never overlap. A recording that can't be transcribed yet (the
/// phone would have to send it to a server and that is not allowed, or
/// speech recognition is not allowed) is left where it is and counted in
/// [VoiceDropsState.waiting]. A phone that cannot recognize a file at all
/// keeps the audio as a note that says so, so nothing recorded is lost.
@injectable
class VoiceDropsBloc extends Bloc<VoiceDropsEvent, VoiceDropsState> {
  VoiceDropsBloc(this._voice, this._dictation, this._settings)
    : super(const VoiceDropsState()) {
    on<VoiceDropsPassRequestedEvent>(_onPass, transformer: droppable());
  }

  final VoiceCaptureRepository _voice;
  final DictationRepository _dictation;
  final VoiceSettingsRepository _settings;

  Future<void> _onPass(
    VoiceDropsPassRequestedEvent event,
    Emitter<VoiceDropsState> emit,
  ) async {
    final listed = await _voice.drops();
    final waiting = listed.getOrElse(() => const <VoiceDrop>[]);
    if (waiting.isEmpty) {
      if (state.waiting != 0) {
        emit(state.copyWith(waiting: 0));
      }
      return;
    }
    emit(state.copyWith(working: true));
    final settings = (await _settings.settings()).fold(
      (error) => null,
      (value) => value,
    );
    final allowServer = settings?.allowServerRecognition ?? false;
    final saved = <String>[];
    final problems = <String>[];
    var waitingCount = 0;
    for (final drop in listed.getOrElse(() => const [])) {
      final heard = await _dictation.transcribeFile(
        drop.audioPath,
        allowServer: allowServer,
      );
      final transcript = heard.fold<FileTranscript?>(
        (error) => null,
        (value) => value,
      );
      if (transcript == null) {
        waitingCount++;
        continue;
      }
      switch (transcript.problem) {
        case null:
          final made = await _voice.saveDrop(
            drop.id,
            transcript.text,
            onDevice: transcript.onDevice,
          );
          made.fold(
            (error) => waitingCount++,
            (result) {
              final id = result.noteId;
              if (id != null) {
                saved.add(id);
                if (result.tidyNow) {
                  unawaited(_voice.tidy(id));
                }
              }
            },
          );
        case FileTranscriptProblem.unsupported:
          final made = await _voice.saveDropAudioOnly(drop.id);
          made.fold(
            (error) => waitingCount++,
            (result) {
              final id = result.noteId;
              if (id != null) {
                saved.add(id);
              }
            },
          );
        case FileTranscriptProblem.notOnDevice ||
            FileTranscriptProblem.denied ||
            FileTranscriptProblem.failed:
          waitingCount++;
        case FileTranscriptProblem.unreadable:
          // A file that can't be opened never will be: do not keep it.
          await _voice.discardDrop(drop.id);
          problems.add(
            'A recording could not be read and was discarded.',
          );
      }
    }
    emit(
      state.copyWith(
        working: false,
        waiting: waitingCount,
        saved: saved,
        savedCount: state.savedCount + saved.length,
        problems: <String>[...state.problems, ...problems],
        tidyMode: settings?.tidy ?? state.tidyMode,
      ),
    );
  }
}
