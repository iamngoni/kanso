// Kanso — share_ready_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../state/share_note/share_note_bloc.dart';
import '../../utils/share_labels.dart';
import 'share_end_section.dart';
import 'share_invite_bar.dart';
import 'share_link_maker.dart';
import 'share_link_tile.dart';
import 'share_made_link.dart';
import 'share_member_row.dart';
import 'share_message_text.dart';
import 'share_prompt_view.dart';
import 'share_request_row.dart';
import 'share_section_header.dart';

/// A note that is shared, or can be: what it is, the invite field for its
/// owner, its people, the people who asked to join, its invite links, and
/// the way to stop sharing or leave. A question about someone's keys takes
/// the place of all that while it is open.
///
/// It needs a [ShareNoteBloc] above it, and shows the state it has.
class ShareReadyView extends StatelessWidget {
  const ShareReadyView({required this.state, super.key});

  final ShareNoteState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final bloc = context.read<ShareNoteBloc>();
    final prompt = state.prompt;
    if (prompt != null) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SharePromptView(prompt: prompt, busy: state.busy),
          if (state.problem != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            ShareMessageText(problem: state.problem),
          ],
        ],
      );
    }
    final info = state.info;
    final manage = state.manages;
    final madeLink = state.madeLink;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        if (info == null)
          Text(l10n.shareIntro, style: typography.body)
        else if (info.myRole != ShareRole.owner)
          Text(
            l10n.shareSharedWithYou(info.owner, l10n.shareAccess(info.myRole)),
            style: typography.body,
          ),
        if (manage) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          const ShareInviteBar(),
        ],
        if (state.done != null || state.problem != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          ShareMessageText(done: state.done, problem: state.problem),
        ],
        if (info != null) ...<Widget>[
          ShareSectionHeader(label: l10n.sharePeople),
          for (final member in info.members)
            ShareMemberRow(
              key: ValueKey<String>('member-${member.accountId}'),
              member: member,
              manage: manage,
              busy: state.busy,
              onRoleChosen: (role) => bloc.add(
                ShareNoteEvent.memberRoleChanged(member.accountId, role),
              ),
              onRemoved: () =>
                  bloc.add(ShareNoteEvent.memberRemoved(member.accountId)),
              onKeyReview: () =>
                  bloc.add(ShareNoteEvent.keyReviewed(member.username)),
            ),
          if (manage && info.requests.isNotEmpty) ...<Widget>[
            ShareSectionHeader(label: l10n.shareRequests),
            for (final request in info.requests)
              ShareRequestRow(
                key: ValueKey<String>('request-${request.accountId}'),
                request: request,
                busy: state.busy,
                onApprove: () =>
                    bloc.add(ShareNoteEvent.requestApproved(request.accountId)),
                onDecline: () =>
                    bloc.add(ShareNoteEvent.requestDeclined(request.accountId)),
              ),
          ],
          if (manage) ...<Widget>[
            ShareSectionHeader(label: l10n.shareLinks),
            for (final link in info.links)
              ShareLinkTile(
                key: ValueKey<String>('link-${link.id}'),
                link: link,
                busy: state.busy,
                onCancel: () => bloc.add(ShareNoteEvent.linkCancelled(link.id)),
              ),
            const SizedBox(height: KansoSpacing.xs),
            const ShareLinkMaker(),
            if (madeLink != null) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              ShareMadeLink(link: madeLink, copied: state.linkCopied),
            ],
          ],
          const SizedBox(height: KansoSpacing.lg),
          ShareEndSection(
            role: info.myRole,
            confirming: state.confirmEnd,
            busy: state.busy,
          ),
        ],
      ],
    );
  }
}
