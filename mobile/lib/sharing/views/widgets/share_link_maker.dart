// Kanso — share_link_maker.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../models/data/share_choices.dart';
import '../../state/share_note/share_note_bloc.dart';
import 'share_role_segmented.dart';

/// The choices for a new invite link (what people who join with it can do,
/// and for how many days) and Make link. It needs a [ShareNoteBloc] above
/// it.
class ShareLinkMaker extends StatelessWidget {
  const ShareLinkMaker({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<ShareNoteBloc, ShareNoteState>(
      buildWhen: (before, after) =>
          before.busy != after.busy ||
          before.linkRole != after.linkRole ||
          before.linkDays != after.linkDays,
      builder: (context, state) {
        final bloc = context.read<ShareNoteBloc>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            ShareRoleSegmented(
              selected: state.linkRole,
              semanticLabel: l10n.shareLinkRoleLabel,
              onChanged: (role) =>
                  bloc.add(ShareNoteEvent.linkRoleChosen(role)),
            ),
            const SizedBox(height: KansoSpacing.xs),
            KansoSegmented<int>(
              expanded: true,
              semanticLabel: l10n.shareLinkDaysLabel,
              selected: state.linkDays,
              onChanged: (days) =>
                  bloc.add(ShareNoteEvent.linkDaysChosen(days)),
              segments: <KansoSegment<int>>[
                for (final days in shareLinkDays)
                  KansoSegment<int>(
                    value: days,
                    label: l10n.shareLinkDays(days),
                  ),
              ],
            ),
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: l10n.shareLinkMake,
              variant: KansoButtonVariant.secondary,
              onPressed: state.busy
                  ? null
                  : () => bloc.add(const ShareNoteEvent.linkMade()),
            ),
          ],
        );
      },
    );
  }
}
