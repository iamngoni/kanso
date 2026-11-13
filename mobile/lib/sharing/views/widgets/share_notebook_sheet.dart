// Kanso — share_notebook_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/shared_notebooks_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../injection.dart';
import '../../models/data/book_prompt.dart';
import '../../models/data/share_sheet_status.dart';
import '../../state/share_notebook/share_notebook_bloc.dart';
import 'book_invite_bar.dart';
import 'book_member_row.dart';
import 'fingerprint_block.dart';
import 'share_message_text.dart';
import 'share_section_header.dart';

/// The Share sheet of the notebook [notebookId]: the desktop's Share
/// notebook sheet. It makes the [ShareNotebookBloc] it shows and gives it
/// up when it closes.
class ShareNotebookSheet extends StatelessWidget {
  const ShareNotebookSheet({required this.notebookId, super.key});

  final String notebookId;

  static Future<void> show(BuildContext context, String notebookId) {
    return showKansoBottomSheet<void>(
      context,
      builder: (context) => ShareNotebookSheet(notebookId: notebookId),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<ShareNotebookBloc>(
      create: (_) =>
          getIt<ShareNotebookBloc>()
            ..add(ShareNotebookEvent.started(notebookId)),
      child: const ShareNotebookView(),
    );
  }
}

/// The content of a notebook's Share sheet: its title, and its sharing or
/// why there is none to show. It needs a [ShareNotebookBloc] above it.
class ShareNotebookView extends StatelessWidget {
  const ShareNotebookView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return BlocBuilder<ShareNotebookBloc, ShareNotebookState>(
      builder: (context, state) {
        final name = state.name.trim();
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
                    child: Text(
                      name.isEmpty
                          ? l10n.shareNotebookTitlePlain
                          : l10n.shareNotebookTitle(name),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: typography.heading2,
                    ),
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
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  KansoSpacing.xs,
                  KansoSpacing.md,
                  KansoSpacing.lg,
                ),
                child: _body(context, state),
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _body(BuildContext context, ShareNotebookState state) {
    final l10n = context.l10n;
    final typography = context.typography;
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
            KansoSkeleton(height: 32),
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
        state.reason ?? l10n.shareNotebookEmpty,
        style: typography.body,
      ),
      ShareSheetStatus.signedOut => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(l10n.shareNotebookSignedOut, style: typography.body),
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
              onPressed: () => context.read<ShareNotebookBloc>().add(
                const ShareNotebookEvent.retried(),
              ),
            ),
          ],
        ),
      ),
      ShareSheetStatus.ready => _Ready(state: state),
    };
  }
}

class _Ready extends StatelessWidget {
  const _Ready({required this.state});

  final ShareNotebookState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final colors = context.colors;
    final bloc = context.read<ShareNotebookBloc>();
    final prompt = state.prompt;
    if (prompt != null) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          _PromptView(prompt: prompt, busy: state.busy),
          if (state.problem != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            ShareMessageText(problem: state.problem),
          ],
        ],
      );
    }
    final info = state.info;
    final check = state.check;
    final ended = state.done;
    if (ended is BookDoneEnded) {
      return Semantics(
        liveRegion: true,
        child: Text(ended.ended.message, style: typography.body),
      );
    }
    final roles =
        info?.inviteRoles ??
        const <BookRole>[
          BookRole.manager,
          BookRole.editor,
          BookRole.commenter,
          BookRole.viewer,
        ];
    final progress = state.progress;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Text(l10n.shareNotebookIntro, style: typography.secondary),
        if (check != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          _Facts(check: check),
        ],
        if (roles.isNotEmpty) ...<Widget>[
          const SizedBox(height: KansoSpacing.md),
          BookInviteBar(roles: roles),
        ],
        if (progress != null && state.busy) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          _Progress(name: state.name, progress: progress),
        ] else if (state.done != null || state.problem != null) ...<Widget>[
          const SizedBox(height: KansoSpacing.sm),
          if (state.problem != null)
            ShareMessageText(problem: state.problem)
          else
            Semantics(
              liveRegion: true,
              child: Text(
                switch (state.done!) {
                  BookDoneShared(:final username) => l10n.shareNotebookShared(
                    state.name,
                    username,
                  ),
                  BookDoneInvited(:final username) => l10n.shareNotebookInvited(
                    username,
                  ),
                  BookDoneRemoved(:final username) => l10n.shareNotebookRemoved(
                    username,
                  ),
                  BookDoneEnded(:final ended) => ended.message,
                },
                style: typography.secondary.copyWith(
                  color: colors.textSecondary,
                ),
              ),
            ),
        ],
        if (info != null) ...<Widget>[
          ShareSectionHeader(label: l10n.sharePeople),
          for (final member in info.members)
            BookMemberRowView(
              key: ValueKey<String>('book-member-${member.accountId}'),
              member: member,
              busy: state.busy,
              onRoleChosen: (role) => bloc.add(
                ShareNotebookEvent.memberRoleChanged(member.accountId, role),
              ),
              onRemove: () => _remove(context, info, member),
            ),
          const SizedBox(height: KansoSpacing.lg),
          KansoButton(
            label: info.owned
                ? l10n.shareNotebookStop
                : l10n.shareNotebookLeave,
            variant: KansoButtonVariant.danger,
            onPressed: state.busy ? null : () => _end(context, info),
          ),
        ],
      ],
    );
  }

  Future<void> _remove(
    BuildContext context,
    BookShareInfo info,
    BookMemberRow member,
  ) async {
    final l10n = context.l10n;
    final bloc = context.read<ShareNotebookBloc>();
    final words = getIt<SharedNotebooksRepository>().words(
      info.name,
      member.username,
    );
    final yes = await KansoConfirmSheet.show(
      context,
      title: words.removeTitle,
      message: words.removeBody,
      confirmLabel: l10n.shareNotebookRemoveConfirm,
    );
    if (yes) {
      bloc.add(ShareNotebookEvent.memberRemoved(member));
    }
  }

  Future<void> _end(BuildContext context, BookShareInfo info) async {
    final l10n = context.l10n;
    final bloc = context.read<ShareNotebookBloc>();
    final words = getIt<SharedNotebooksRepository>().words(info.name, '');
    final yes = await KansoConfirmSheet.show(
      context,
      title: info.owned ? words.stopTitle : words.leaveTitle,
      message: info.owned
          ? l10n.shareNotebookStopBody
          : l10n.shareNotebookLeaveBody,
      confirmLabel: info.owned
          ? l10n.shareNotebookStop
          : l10n.shareNotebookLeave,
    );
    if (yes) {
      bloc.add(const ShareNotebookEvent.ended());
    }
  }
}

/// What sharing the notebook would do: how many notes go in, the locked
/// ones that stay out, and the notes left out.
class _Facts extends StatelessWidget {
  const _Facts({required this.check});

  final BookShareCheck check;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final lines = <(KansoIcons, String)>[
      (KansoIcons.fileText, l10n.shareNotebookNotes(check.notes)),
      if (check.locked > 0)
        (
          KansoIcons.lock,
          l10n.shareNotebookLocked(check.locked, check.lockedName),
        ),
      if (check.leftOut.isNotEmpty)
        (KansoIcons.info, l10n.shareNotebookLeftOut(check.leftOut.join(', '))),
    ];
    return Container(
      padding: const EdgeInsets.all(KansoSpacing.sm),
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: KansoRadii.mediumBorder,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          for (final (index, (icon, text)) in lines.indexed) ...<Widget>[
            if (index > 0) const SizedBox(height: KansoSpacing.xs),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                KansoIcon(icon, size: 16, color: colors.textSecondary),
                const SizedBox(width: KansoSpacing.xs),
                Expanded(child: Text(text, style: typography.secondary)),
              ],
            ),
          ],
        ],
      ),
    );
  }
}

/// "Sharing Studio… 120 of 340", with a bar that fills as the notes go up.
class _Progress extends StatelessWidget {
  const _Progress({required this.name, required this.progress});

  final String name;
  final BookShareProgress progress;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final total = progress.total == 0 ? 1 : progress.total;
    return Semantics(
      liveRegion: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(
            context.l10n.shareNotebookSharing(
              name,
              progress.done,
              progress.total,
            ),
            style: context.typography.secondary,
          ),
          const SizedBox(height: KansoSpacing.xs),
          ClipRRect(
            borderRadius: KansoRadii.pillBorder,
            child: LinearProgressIndicator(
              value: progress.done / total,
              minHeight: 4,
              color: colors.accent,
              backgroundColor: colors.fill,
            ),
          ),
        ],
      ),
    );
  }
}

/// A question before someone is invited: compare a new person's fingerprint,
/// or look at keys that changed.
class _PromptView extends StatelessWidget {
  const _PromptView({required this.prompt, required this.busy});

  final BookPrompt prompt;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final bloc = context.read<ShareNotebookBloc>();
    void dismiss() => bloc.add(const ShareNotebookEvent.promptDismissed());
    return switch (prompt) {
      BookPromptVerify(:final person) => Column(
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
                : () =>
                      bloc.add(const ShareNotebookEvent.fingerprintConfirmed()),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.shareInviteUnverified,
            variant: KansoButtonVariant.secondary,
            onPressed: busy
                ? null
                : () => bloc.add(const ShareNotebookEvent.invitedUnverified()),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.shareBack,
            variant: KansoButtonVariant.quiet,
            onPressed: busy ? null : dismiss,
          ),
        ],
      ),
      BookPromptKeyChanged(:final offered, :final pinned) => Column(
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
                : () => bloc.add(const ShareNotebookEvent.newKeyAccepted()),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.shareKeyDontInvite,
            variant: KansoButtonVariant.quiet,
            onPressed: busy ? null : dismiss,
          ),
        ],
      ),
    };
  }
}
