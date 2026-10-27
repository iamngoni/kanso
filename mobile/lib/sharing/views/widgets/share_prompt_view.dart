// Kanso — share_prompt_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/share_prompt.dart';
import '../../state/share_note/share_note_bloc.dart';
import 'fingerprint_block.dart';

/// A question before someone's keys are trusted: the first share with a
/// person asks to compare their fingerprint; keys that changed show the
/// fingerprint before and now, with a warning. It needs a [ShareNoteBloc]
/// above it.
class SharePromptView extends StatelessWidget {
  const SharePromptView({required this.prompt, required this.busy, super.key});

  final SharePrompt prompt;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final bloc = context.read<ShareNoteBloc>();
    void dismiss() => bloc.add(const ShareNoteEvent.promptDismissed());
    return switch (prompt) {
      SharePromptVerify(:final person) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(l10n.shareVerifyIntro(person.username), style: typography.body),
          const SizedBox(height: KansoSpacing.sm),
          FingerprintBlock(
            fingerprint: person.fingerprint,
            label: l10n.shareFingerprintLabel(person.username),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: l10n.shareVerifyConfirm,
            onPressed: busy
                ? null
                : () => bloc.add(const ShareNoteEvent.fingerprintConfirmed()),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.shareInviteUnverified,
            variant: KansoButtonVariant.secondary,
            onPressed: busy
                ? null
                : () => bloc.add(const ShareNoteEvent.invitedUnverified()),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.shareBack,
            variant: KansoButtonVariant.quiet,
            onPressed: busy ? null : dismiss,
          ),
        ],
      ),
      SharePromptKeyChanged(:final offered, :final pinned, :final role) =>
        Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Container(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              decoration: BoxDecoration(
                color: colors.warning.withValues(alpha: KansoEffects.wash),
                borderRadius: KansoRadii.mediumBorder,
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  KansoIcon(KansoIcons.shieldAlert, color: colors.warning),
                  const SizedBox(width: KansoSpacing.sm),
                  Expanded(
                    child: Text(
                      l10n.shareKeyChangedIntro(offered.username),
                      style: typography.body,
                    ),
                  ),
                ],
              ),
            ),
            if (pinned != null) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              Text(l10n.shareKeyBefore, style: typography.sectionLabel),
              const SizedBox(height: KansoSpacing.xxs),
              FingerprintBlock(
                fingerprint: pinned.fingerprint,
                label: l10n.shareKeyBefore,
              ),
            ],
            const SizedBox(height: KansoSpacing.sm),
            Text(l10n.shareKeyNow, style: typography.sectionLabel),
            const SizedBox(height: KansoSpacing.xxs),
            FingerprintBlock(
              fingerprint: offered.fingerprint,
              label: l10n.shareKeyNow,
            ),
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: l10n.shareKeyTrust,
              onPressed: busy
                  ? null
                  : () => bloc.add(const ShareNoteEvent.newKeyAccepted()),
            ),
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: role == null
                  ? l10n.shareKeyNotNow
                  : l10n.shareKeyDontInvite,
              variant: KansoButtonVariant.quiet,
              onPressed: busy ? null : dismiss,
            ),
          ],
        ),
    };
  }
}
