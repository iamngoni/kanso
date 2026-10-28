// Kanso — voice_note_body.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/dictation_status.dart';
import '../../models/data/dictation_unavailable_reason.dart';
import '../../models/data/voice_note_status.dart';
import '../../state/dictation/dictation_bloc.dart';
import '../../state/voice_note/voice_note_bloc.dart';
import '../../utils/leave_voice_note.dart';
import 'dictation_denied_card.dart';
import 'dictation_error_card.dart';
import 'dictation_mic_button.dart';
import 'dictation_unavailable_card.dart';
import 'live_transcript.dart';
import 'voice_note_save_failed_card.dart';

/// What the voice note shows below its bar: the transcript and the mic, or a
/// card when dictation cannot run or the note could not be saved.
class VoiceNoteBody extends StatelessWidget {
  const VoiceNoteBody({super.key});

  @override
  Widget build(BuildContext context) {
    final dictation = context.watch<DictationBloc>().state;
    final saving = context.watch<VoiceNoteBloc>().state;
    final bloc = context.read<DictationBloc>();
    if (saving.status == VoiceNoteStatus.failed) {
      return Center(
        child: VoiceNoteSaveFailedCard(
          onRetry: () => context.read<VoiceNoteBloc>().add(
            VoiceNoteEvent.saveRequested(dictation.text),
          ),
        ),
      );
    }
    switch (dictation.status) {
      case DictationStatus.unavailable:
        return Center(
          child: DictationUnavailableCard(
            reason: dictation.reason ?? DictationUnavailableReason.noRecognizer,
            onClose: () => leaveVoiceNote(context),
          ),
        );
      case DictationStatus.denied:
        return Center(
          child: DictationDeniedCard(
            restricted: dictation.restricted,
            onOpenSettings: () =>
                bloc.add(const DictationEvent.settingsRequested()),
          ),
        );
      case DictationStatus.error when dictation.segments.isEmpty:
        return Center(
          child: DictationErrorCard(
            failure: dictation.failure!,
            onRetry: () => bloc.add(const DictationEvent.started()),
          ),
        );
      case DictationStatus.idle:
      case DictationStatus.requesting:
      case DictationStatus.listening:
      case DictationStatus.error:
        break;
    }
    final listening = dictation.status == DictationStatus.listening;
    final busy =
        dictation.status == DictationStatus.requesting ||
        saving.status == VoiceNoteStatus.saving;
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: KansoSizes.sheetWidth),
        child: Column(
          children: <Widget>[
            Expanded(
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: LiveTranscript(
                  finished: dictation.finishedText,
                  text: dictation.text,
                ),
              ),
            ),
            DictationMicButton(
              listening: listening,
              level: dictation.level,
              onPressed: busy
                  ? null
                  : () => bloc.add(
                      listening
                          ? const DictationEvent.stopped()
                          : const DictationEvent.started(),
                    ),
            ),
            Padding(
              padding: EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.xs,
                KansoSpacing.md,
                KansoSpacing.lg + MediaQuery.paddingOf(context).bottom,
              ),
              child: Text(
                dictation.status == DictationStatus.error
                    ? context.l10n.dictationErrorMessage
                    : listening
                    ? context.l10n.dictationListening
                    : context.l10n.dictationVoiceNoteHint,
                textAlign: TextAlign.center,
                style: context.typography.secondary,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
