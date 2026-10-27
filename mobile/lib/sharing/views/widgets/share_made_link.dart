// Kanso — share_made_link.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../state/share_note/share_note_bloc.dart';
import '../../utils/origin_of.dart';

/// The invite link just made, shown once: its address, a Copy button and a
/// Send button that offers it to the system's share sheet, and a line that
/// says what the link does. It needs a [ShareNoteBloc] above it.
class ShareMadeLink extends StatelessWidget {
  const ShareMadeLink({required this.link, required this.copied, super.key});

  final String link;
  final bool copied;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bloc = context.read<ShareNoteBloc>();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Semantics(
          label: l10n.shareLinkAddress,
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            decoration: BoxDecoration(
              color: colors.list,
              borderRadius: KansoRadii.mediumBorder,
              border: Border.all(color: colors.divider),
            ),
            child: Text(
              link,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.monoSmall,
            ),
          ),
        ),
        const SizedBox(height: KansoSpacing.xs),
        Row(
          children: <Widget>[
            Expanded(
              child: KansoButton(
                label: copied ? l10n.shareLinkCopied : l10n.shareLinkCopy,
                icon: copied ? KansoIcons.check : KansoIcons.copy,
                variant: KansoButtonVariant.secondary,
                onPressed: () => bloc.add(const ShareNoteEvent.linkCopied()),
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            Expanded(
              child: Builder(
                builder: (buttonContext) {
                  return KansoButton(
                    label: l10n.shareLinkSend,
                    icon: KansoIcons.share,
                    variant: KansoButtonVariant.secondary,
                    onPressed: () => bloc.add(
                      ShareNoteEvent.linkSent(origin: originOf(buttonContext)),
                    ),
                  );
                },
              ),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.xs),
        Text(l10n.shareLinkNote, style: context.typography.meta),
      ],
    );
  }
}
