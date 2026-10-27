// Kanso — shared_with_you_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/invitations/invitations_bloc.dart';
import 'invitation_tile.dart';
import 'share_message_text.dart';

/// Shared with you: the notes other people shared with this account, each
/// with Accept and Decline, and the way to join a note with a link. It reads
/// the app's `InvitationsBloc`.
///
/// Accepting a note opens it.
class SharedWithYouPane extends StatelessWidget {
  const SharedWithYouPane({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return BlocListener<InvitationsBloc, InvitationsState>(
      listenWhen: (before, after) =>
          before.opened != after.opened && after.opened != null,
      listener: (context, state) => context.go(
        BrowseLocation.place(
          const ListPlace.all(),
          noteId: state.opened,
        ).location,
      ),
      child: ColoredBox(
        color: colors.app,
        child: Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              bordered: false,
              leading: const ShellBackButton(),
              title: l10n.sharedWithYouTitle,
              trailing: <Widget>[
                KansoIconButton(
                  icon: KansoIcons.refreshCw,
                  semanticLabel: l10n.sharedWithYouRefresh,
                  onPressed: () => context.read<InvitationsBloc>().add(
                    const InvitationsEvent.refreshed(),
                  ),
                ),
              ],
            ),
            Expanded(
              child: BlocBuilder<InvitationsBloc, InvitationsState>(
                builder: (context, state) {
                  return switch (state.status) {
                    LoadStatus.loading => Semantics(
                      label: l10n.sharedWithYouLoading,
                      container: true,
                      child: const Padding(
                        padding: EdgeInsets.all(KansoSpacing.md),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: <Widget>[
                            KansoSkeleton(height: 14),
                            SizedBox(height: KansoSpacing.xs),
                            KansoSkeleton(width: 160),
                            SizedBox(height: KansoSpacing.lg),
                            KansoSkeleton(height: 14),
                            SizedBox(height: KansoSpacing.xs),
                            KansoSkeleton(width: 120),
                          ],
                        ),
                      ),
                    ),
                    LoadStatus.failed => Center(
                      child: KansoMessageCard(
                        title: l10n.sharedWithYouFailedTitle,
                        message: l10n.sharedWithYouFailedMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => context
                                .read<InvitationsBloc>()
                                .add(const InvitationsEvent.refreshed()),
                          ),
                        ],
                      ),
                    ),
                    LoadStatus.ready when state.signedOut => KansoEmptyState(
                      title: l10n.sharedWithYouSignedOutTitle,
                      message: l10n.sharedWithYouSignedOutMessage,
                      action: KansoButton(
                        label: l10n.shareSignIn,
                        expanded: false,
                        onPressed: () => context.push(KansoRoutes.settings),
                      ),
                    ),
                    LoadStatus.ready when state.invitations.isEmpty =>
                      KansoEmptyState(
                        title: l10n.sharedWithYouEmptyTitle,
                        message: l10n.sharedWithYouEmptyMessage,
                        action: KansoButton(
                          label: l10n.sharedJoinWithLink,
                          variant: KansoButtonVariant.secondary,
                          icon: KansoIcons.link,
                          expanded: false,
                          onPressed: () => context.push(KansoRoutes.sharedJoin),
                        ),
                      ),
                    LoadStatus.ready => ListView(
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      children: <Widget>[
                        KansoGroupedSection(
                          children: <Widget>[
                            for (final invitation in state.invitations)
                              InvitationTile(
                                key: ValueKey<String>(invitation.shareId),
                                invitation: invitation,
                                working: state.working.contains(
                                  invitation.shareId,
                                ),
                                onAccept: () =>
                                    context.read<InvitationsBloc>().add(
                                      InvitationsEvent.accepted(
                                        invitation.shareId,
                                      ),
                                    ),
                                onDecline: () =>
                                    context.read<InvitationsBloc>().add(
                                      InvitationsEvent.declined(
                                        invitation.shareId,
                                      ),
                                    ),
                              ),
                          ],
                        ),
                        if (state.problem != null) ...<Widget>[
                          const SizedBox(height: KansoSpacing.sm),
                          ShareMessageText(problem: state.problem),
                        ],
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.link,
                              label: l10n.sharedJoinWithLink,
                              chevron: true,
                              onTap: () => context.push(KansoRoutes.sharedJoin),
                            ),
                          ],
                        ),
                      ],
                    ),
                  };
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
