// Kanso — share_note_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../models/data/share_sheet_status.dart';
import '../../state/share_note/share_note_bloc.dart';
import 'share_ready_view.dart';

/// The content of a note's share sheet: a title with a close button, and
/// what the note's sharing is, or why there is none to show. It needs a
/// [ShareNoteBloc] above it, and is made for a sheet that is as tall as it
/// needs and scrolls when it can't be.
class ShareNoteView extends StatelessWidget {
  const ShareNoteView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final keyboard = MediaQuery.viewInsetsOf(context).bottom;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.md,
            end: KansoSpacing.xs,
            top: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: Text(l10n.shareTitle, style: typography.heading2),
              ),
              KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.shareClose,
                onPressed: () => Navigator.of(context).maybePop(),
              ),
            ],
          ),
        ),
        Flexible(
          child: SingleChildScrollView(
            padding: EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.lg + keyboard,
            ),
            child: BlocBuilder<ShareNoteBloc, ShareNoteState>(
              builder: (context, state) {
                return switch (state.status) {
                  ShareSheetStatus.loading => Semantics(
                    label: l10n.shareLoading,
                    container: true,
                    child: const Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: <Widget>[
                        KansoSkeleton(height: 14),
                        SizedBox(height: KansoSpacing.xs),
                        KansoSkeleton(width: 220, height: 14),
                        SizedBox(height: KansoSpacing.lg),
                        KansoSkeleton(height: 44),
                        SizedBox(height: KansoSpacing.lg),
                        KansoSkeleton(width: 120),
                        SizedBox(height: KansoSpacing.sm),
                        KansoSkeleton(height: 32),
                      ],
                    ),
                  ),
                  ShareSheetStatus.locked => Text(
                    l10n.shareLockedNote,
                    style: typography.body,
                  ),
                  ShareSheetStatus.notShareable => Text(
                    state.reason ?? '',
                    style: typography.body,
                  ),
                  ShareSheetStatus.signedOut => Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      Text(l10n.shareSignedOut, style: typography.body),
                      const SizedBox(height: KansoSpacing.md),
                      KansoButton(
                        label: l10n.shareSignIn,
                        onPressed: () {
                          final router = GoRouter.of(context);
                          Navigator.of(context).pop();
                          router.push(KansoRoutes.settings);
                        },
                      ),
                    ],
                  ),
                  ShareSheetStatus.failed => Center(
                    child: KansoMessageCard(
                      title: l10n.shareFailedTitle,
                      message: l10n.shareFailedMessage,
                      actions: <Widget>[
                        KansoButton(
                          label: l10n.retry,
                          onPressed: () => context.read<ShareNoteBloc>().add(
                            const ShareNoteEvent.retried(),
                          ),
                        ),
                      ],
                    ),
                  ),
                  ShareSheetStatus.ready => ShareReadyView(state: state),
                };
              },
            ),
          ),
        ),
      ],
    );
  }
}
