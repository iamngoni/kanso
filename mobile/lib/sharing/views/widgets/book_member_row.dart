// Kanso — book_member_row.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../utils/book_labels.dart';
import 'book_avatars.dart';

/// A person in a shared notebook: their face, their name, what is said
/// under it ("Invited, hasn't accepted yet", "You") and their role.
///
/// When the Rust core offers roles for them ([BookMemberRow.roles]) or lets
/// this account remove them, the role is a button that opens their menu.
class BookMemberRowView extends StatelessWidget {
  const BookMemberRowView({
    required this.member,
    required this.busy,
    required this.onRoleChosen,
    required this.onRemove,
    super.key,
  });

  final BookMemberRow member;
  final bool busy;
  final ValueChanged<BookRole> onRoleChosen;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final gone = member.standing == BookStandingRow.left;
    final notes = <String>[
      if (member.standing == BookStandingRow.invited) l10n.bookMemberInvited,
      if (gone) l10n.bookMemberLeft,
      if (member.me) l10n.shareMemberYou,
    ].join(' · ');
    final menu = member.roles.isNotEmpty || member.removable;
    final role = bookRoleLabel(l10n, member.role);
    return Opacity(
      opacity: gone ? 0.5 : 1,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xs),
        child: Row(
          children: <Widget>[
            BookAvatars(
              names: <String>[member.username],
              size: 32,
              ring: colors.elevated,
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Row(
                    children: <Widget>[
                      Flexible(
                        child: Text(
                          member.username,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body,
                        ),
                      ),
                      if (member.verified && !member.me) ...<Widget>[
                        const SizedBox(width: KansoSpacing.xxs),
                        KansoIcon(
                          KansoIcons.shieldCheck,
                          size: 14,
                          color: colors.textMuted,
                        ),
                      ],
                    ],
                  ),
                  if (notes.isNotEmpty) Text(notes, style: typography.meta),
                ],
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            if (menu)
              KansoButton(
                label: role,
                variant: KansoButtonVariant.secondary,
                expanded: false,
                compact: true,
                onPressed: busy
                    ? null
                    : () => BookMemberMenu.show(
                        context,
                        member: member,
                        onRoleChosen: onRoleChosen,
                        onRemove: onRemove,
                      ),
              )
            else
              Text(role, style: typography.secondary),
          ],
        ),
      ),
    );
  }
}

/// A member's menu: the roles the Rust core offers them, the current one
/// ticked, and Remove from notebook… when it lets this account remove them.
class BookMemberMenu extends StatelessWidget {
  const BookMemberMenu({
    required this.member,
    required this.onRoleChosen,
    required this.onRemove,
    super.key,
  });

  final BookMemberRow member;
  final ValueChanged<BookRole> onRoleChosen;
  final VoidCallback onRemove;

  /// Shows [member]'s menu. The choice runs after the menu has closed.
  static Future<void> show(
    BuildContext context, {
    required BookMemberRow member,
    required ValueChanged<BookRole> onRoleChosen,
    required VoidCallback onRemove,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (context) => BookMemberMenu(
        member: member,
        onRoleChosen: onRoleChosen,
        onRemove: onRemove,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    void choose(VoidCallback then) {
      Navigator.of(context).pop();
      then();
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            member.username,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: context.typography.heading2,
          ),
        ),
        for (final role in member.roles)
          KansoSettingsRow(
            label: bookRoleLabel(l10n, role),
            trailing: role == member.role
                ? KansoIcon(KansoIcons.check, color: colors.accent)
                : null,
            onTap: () => choose(() {
              if (role != member.role) {
                onRoleChosen(role);
              }
            }),
          ),
        if (member.removable) ...<Widget>[
          if (member.roles.isNotEmpty) const KansoDivider(),
          KansoSettingsRow(
            label: l10n.shareNotebookRemoveMember,
            danger: true,
            onTap: () => choose(onRemove),
          ),
        ],
        const SizedBox(height: KansoSpacing.xs),
      ],
    );
  }
}
