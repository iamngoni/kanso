// Kanso — shared_note_status_strip.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../state/shared_note_status/shared_note_status_bloc.dart';
import '../../utils/share_labels.dart';

/// Under a shared note's bar: what couldn't be done for it, on a warning
/// wash with Dismiss, and its status line ("Shared · Can view · Read only ·
/// Live"), as the desktop's banner and footer say them. A note that isn't
/// shared shows nothing. It reads the `SharedNoteStatusBloc` above it.
class SharedNoteStatusStrip extends StatelessWidget {
  const SharedNoteStatusStrip({required this.gutter, super.key});

  /// The space either side, the page's.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<SharedNoteStatusBloc, SharedNoteStatusState>(
      buildWhen: (before, now) => before.status != now.status,
      builder: (context, state) {
        final status = state.status;
        if (status == null) {
          return const SizedBox.shrink();
        }
        final l10n = context.l10n;
        final colors = context.colors;
        final typography = context.typography;
        final banner = l10n.sharedProblemsBanner(status.problems);
        return Padding(
          padding: EdgeInsets.fromLTRB(gutter, 0, gutter, KansoSpacing.xxs),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              if (banner != null)
                Semantics(
                  liveRegion: true,
                  child: Container(
                    margin: const EdgeInsets.only(bottom: KansoSpacing.xxs),
                    padding: const EdgeInsetsDirectional.only(
                      start: KansoSpacing.sm,
                      end: KansoSpacing.xxs,
                      top: KansoSpacing.xxs,
                      bottom: KansoSpacing.xxs,
                    ),
                    decoration: BoxDecoration(
                      color: colors.warning.withValues(alpha: 0.14),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Row(
                      children: <Widget>[
                        Expanded(
                          child: Text(banner, style: typography.secondary),
                        ),
                        KansoButton(
                          label: l10n.sharedProblemsDismiss,
                          variant: KansoButtonVariant.quiet,
                          expanded: false,
                          compact: true,
                          onPressed: () =>
                              context.read<SharedNoteStatusBloc>().add(
                                const SharedNoteStatusEvent.problemsDismissed(),
                              ),
                        ),
                      ],
                    ),
                  ),
                ),
              Row(
                children: <Widget>[
                  KansoIcon(
                    KansoIcons.users,
                    size: KansoSizes.iconSmall,
                    color: colors.textMuted,
                  ),
                  const SizedBox(width: KansoSpacing.xxs),
                  Expanded(
                    child: Text(
                      l10n.sharedStatusLine(status),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta.copyWith(color: colors.textMuted),
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}
