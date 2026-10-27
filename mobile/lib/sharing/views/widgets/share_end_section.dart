// Kanso — share_end_section.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../state/share_note/share_note_bloc.dart';

/// Stop sharing (for the owner) or Leave (for anyone else), which asks once
/// before it does it. It needs a [ShareNoteBloc] above it.
class ShareEndSection extends StatelessWidget {
  const ShareEndSection({
    required this.role,
    required this.confirming,
    required this.busy,
    super.key,
  });

  /// This account's role in the note.
  final ShareRole role;
  final bool confirming;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<ShareNoteBloc>();
    final owner = role == ShareRole.owner;
    if (!confirming) {
      return KansoButton(
        label: owner ? l10n.shareStop : l10n.shareLeave,
        variant: KansoButtonVariant.danger,
        onPressed: busy
            ? null
            : () => bloc.add(const ShareNoteEvent.endAsked()),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Text(
          owner ? l10n.shareStopWarning : l10n.shareLeaveWarning,
          style: context.typography.body.copyWith(color: context.colors.danger),
        ),
        const SizedBox(height: KansoSpacing.sm),
        KansoButton(
          label: owner ? l10n.shareStopEveryone : l10n.shareLeaveNote,
          variant: KansoButtonVariant.danger,
          onPressed: busy
              ? null
              : () => bloc.add(const ShareNoteEvent.endConfirmed()),
        ),
        const SizedBox(height: KansoSpacing.xs),
        KansoButton(
          label: l10n.shareKeepSharing,
          variant: KansoButtonVariant.quiet,
          onPressed: busy
              ? null
              : () => bloc.add(const ShareNoteEvent.endDismissed()),
        ),
      ],
    );
  }
}
