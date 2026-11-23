// Kanso — feedback_credit_picker.dart
// Created 2026-11-23.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/feedback_credit.dart';
import '../../state/send_feedback/send_feedback_bloc.dart';
import 'feedback_github_avatar.dart';

/// The two ways to be credited, and the field for the one chosen: a plain
/// name, or a GitHub username picked from a search (the issue then mentions
/// them, so GitHub notifies them). It needs a [SendFeedbackBloc] above it.
class FeedbackCreditPicker extends StatelessWidget {
  const FeedbackCreditPicker({
    required this.name,
    required this.search,
    super.key,
  });

  /// The plain-name field's controller.
  final TextEditingController name;

  /// The GitHub search field's controller.
  final TextEditingController search;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SendFeedbackBloc>();
    return BlocBuilder<SendFeedbackBloc, SendFeedbackState>(
      buildWhen: (before, after) =>
          before.draft.creditKind != after.draft.creditKind ||
          before.draft.githubUser != after.draft.githubUser ||
          before.draft.creditName != after.draft.creditName ||
          before.githubSearch != after.githubSearch ||
          before.sending != after.sending,
      builder: (context, state) {
        final draft = state.draft;
        return Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            0,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              KansoSegmented<FeedbackCreditKind>(
                expanded: true,
                semanticLabel: l10n.feedbackCreditKindLabel,
                selected: draft.creditKind,
                segments: <KansoSegment<FeedbackCreditKind>>[
                  KansoSegment<FeedbackCreditKind>(
                    value: FeedbackCreditKind.name,
                    label: l10n.feedbackCreditKindName,
                  ),
                  KansoSegment<FeedbackCreditKind>(
                    value: FeedbackCreditKind.github,
                    label: l10n.feedbackCreditKindGithub,
                  ),
                ],
                onChanged: state.sending
                    ? (_) {}
                    : (kind) =>
                          bloc.add(SendFeedbackEvent.creditKindChanged(kind)),
              ),
              const SizedBox(height: KansoSpacing.sm),
              if (draft.creditKind == FeedbackCreditKind.name) ...<Widget>[
                KansoTextField(
                  controller: name,
                  hint: l10n.feedbackCreditName,
                  semanticLabel: l10n.feedbackCreditName,
                  invalid:
                      draft.creditName.isNotEmpty && !draft.creditNameValid,
                  enabled: !state.sending,
                  onChanged: (value) =>
                      bloc.add(SendFeedbackEvent.creditNameChanged(value)),
                ),
                _Detail(text: l10n.feedbackCreditNameDetail),
              ] else ...<Widget>[
                if (draft.githubUser case final user?)
                  _PickedUser(user: user, enabled: !state.sending)
                else ...<Widget>[
                  KansoTextField(
                    controller: search,
                    hint: l10n.feedbackGithubSearchHint,
                    semanticLabel: l10n.feedbackGithubSearchHint,
                    enabled: !state.sending,
                    onChanged: (value) => bloc.add(
                      SendFeedbackEvent.githubQueryChanged(value),
                    ),
                  ),
                  _Results(search: state.githubSearch),
                ],
                _Detail(text: l10n.feedbackCreditGithubDetail),
              ],
            ],
          ),
        );
      },
    );
  }
}

/// A line of explanation under the field.
class _Detail extends StatelessWidget {
  const _Detail({required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.xs),
      child: Text(
        text,
        style: context.typography.secondary.copyWith(
          color: context.colors.textSecondary,
        ),
      ),
    );
  }
}

/// The users found, or what the search says instead.
class _Results extends StatelessWidget {
  const _Results({required this.search});

  final FeedbackGithubSearch search;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return switch (search) {
      FeedbackGithubSearchIdle() => const SizedBox.shrink(),
      FeedbackGithubSearchSearching() => _Detail(
        text: l10n.feedbackGithubSearching,
      ),
      FeedbackGithubSearchUnavailable() => _Detail(
        text: l10n.feedbackGithubUnavailable,
      ),
      FeedbackGithubSearchFound(:final users) when users.isEmpty => _Detail(
        text: l10n.feedbackGithubNoneFound,
      ),
      FeedbackGithubSearchFound(:final users) => Padding(
        padding: const EdgeInsets.only(top: KansoSpacing.xs),
        child: DecoratedBox(
          decoration: BoxDecoration(
            border: Border.all(color: colors.fill),
            borderRadius: KansoRadii.mediumBorder,
          ),
          child: Column(
            children: <Widget>[
              for (final user in users)
                _UserRow(
                  user: user,
                  onTap: () => context.read<SendFeedbackBloc>().add(
                    SendFeedbackEvent.githubUserPicked(user),
                  ),
                ),
            ],
          ),
        ),
      ),
    };
  }
}

/// One result: avatar and login, a full-width touch target.
class _UserRow extends StatelessWidget {
  const _UserRow({required this.user, required this.onTap});

  final FeedbackGithubUser user;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: context.l10n.feedbackGithubUserSemantics(user.login),
      child: InkWell(
        onTap: onTap,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: 44),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xxs,
            ),
            child: Row(
              children: <Widget>[
                FeedbackGithubAvatar(login: user.login, url: user.avatarUrl),
                const SizedBox(width: KansoSpacing.sm),
                Expanded(
                  child: Text(
                    user.login,
                    style: context.typography.body,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// The chosen user, with Change to search again.
class _PickedUser extends StatelessWidget {
  const _PickedUser({required this.user, required this.enabled});

  final FeedbackGithubUser user;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        FeedbackGithubAvatar(login: user.login, url: user.avatarUrl),
        const SizedBox(width: KansoSpacing.sm),
        Expanded(
          child: Text(
            '@${user.login}',
            style: context.typography.body,
            overflow: TextOverflow.ellipsis,
          ),
        ),
        KansoButton(
          label: context.l10n.feedbackGithubChange,
          variant: KansoButtonVariant.quiet,
          expanded: false,
          compact: true,
          onPressed: enabled
              ? () => context.read<SendFeedbackBloc>().add(
                  const SendFeedbackEvent.githubUserCleared(),
                )
              : null,
        ),
      ],
    );
  }
}
