// Kanso — share_member_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../utils/member_status.dart';
import '../../utils/share_labels.dart';
import 'share_member_menu.dart';

/// A person in a shared note: an icon that says whether their keys are
/// checked, their name, what is said under it, and their role.
///
/// When the person [manage]s the note and this is not their own row, the role
/// is a button that opens a menu to change it or remove the person. A person
/// whose keys changed has a warning and the action to review the new key.
class ShareMemberRow extends StatelessWidget {
  const ShareMemberRow({
    required this.member,
    required this.manage,
    required this.busy,
    required this.onRoleChosen,
    required this.onRemoved,
    required this.onKeyReview,
    super.key,
  });

  final MemberRow member;

  /// Whether this account may change roles and remove people.
  final bool manage;

  /// A step is running: nothing can be pressed.
  final bool busy;
  final ValueChanged<ShareRole> onRoleChosen;
  final VoidCallback onRemoved;
  final VoidCallback onKeyReview;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final typography = context.typography;
    final status = memberStatus(member);
    final tone = status.tone;
    final icon = switch (tone) {
      MemberTone.unchecked => KansoIcons.shieldAlert,
      _ when member.verified || member.me => KansoIcons.shieldCheck,
      _ => KansoIcons.circleUser,
    };
    final notes = status.notes.map(l10n.shareMemberNoteText).join(' · ');
    final changeable =
        manage &&
        member.role != ShareRole.owner &&
        !member.me &&
        tone != MemberTone.gone;
    final role = changeable
        ? KansoButton(
            label: l10n.shareRoleName(member.role),
            variant: KansoButtonVariant.secondary,
            expanded: false,
            compact: true,
            onPressed: busy
                ? null
                : () => ShareMemberMenu.show(
                    context,
                    member: member,
                    onRoleChosen: onRoleChosen,
                    onRemoved: onRemoved,
                  ),
          )
        : Text(l10n.shareRoleName(member.role), style: typography.secondary);
    return Opacity(
      opacity: tone == MemberTone.gone ? 0.5 : 1,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.only(top: KansoSpacing.xs),
              child: KansoIcon(
                icon,
                color: tone == MemberTone.unchecked
                    ? colors.warning
                    : colors.textSecondary,
              ),
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xs),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(
                      member.username,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.body,
                    ),
                    if (notes.isNotEmpty)
                      Text(
                        notes,
                        style: typography.meta.copyWith(
                          color: tone == MemberTone.unchecked
                              ? colors.warning
                              : null,
                        ),
                      ),
                    if (tone == MemberTone.unchecked)
                      Padding(
                        padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                        child: KansoButton(
                          label: l10n.shareMemberAcceptKey,
                          variant: KansoButtonVariant.quiet,
                          expanded: false,
                          compact: true,
                          onPressed: busy ? null : onKeyReview,
                        ),
                      ),
                  ],
                ),
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            role,
          ],
        ),
      ),
    );
  }
}
