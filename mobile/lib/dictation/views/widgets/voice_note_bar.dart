// Kanso — voice_note_bar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/voice_note_status.dart';
import '../../state/dictation/dictation_bloc.dart';
import '../../state/voice_note/voice_note_bloc.dart';
import '../../utils/leave_voice_note.dart';

/// The bar of the voice note: Cancel, the title and Done. Done waits for
/// something to have been heard.
class VoiceNoteBar extends StatelessWidget {
  const VoiceNoteBar({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final dictation = context.watch<DictationBloc>().state;
    final saving =
        context.watch<VoiceNoteBloc>().state.status == VoiceNoteStatus.saving;
    final canFinish =
        dictation.text.trim().isNotEmpty && !saving && !dictation.finished;
    return KansoTopBar(
      leading: KansoButton(
        label: l10n.dictationCancel,
        variant: KansoButtonVariant.quiet,
        expanded: false,
        compact: true,
        onPressed: () {
          context.read<DictationBloc>().add(const DictationEvent.cancelled());
          leaveVoiceNote(context);
        },
      ),
      title: l10n.dictationVoiceNoteTitle,
      trailing: <Widget>[
        KansoButton(
          label: l10n.dictationDone,
          expanded: false,
          compact: true,
          onPressed: canFinish
              ? () => context.read<DictationBloc>().add(
                  const DictationEvent.finishRequested(),
                )
              : null,
        ),
      ],
    );
  }
}
