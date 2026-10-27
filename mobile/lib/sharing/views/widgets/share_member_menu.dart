// Kanso — share_member_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../models/data/share_choices.dart';
import '../../utils/share_labels.dart';

/// What can be done with a person in a shared note: give them another role
/// or remove them. A sheet over the share sheet, with the roles and a check
/// on the one they have.
class ShareMemberMenu extends StatelessWidget {
  const ShareMemberMenu({
    required this.member,
    required this.onRoleChosen,
    required this.onRemoved,
    super.key,
  });

  final MemberRow member;
  final ValueChanged<ShareRole> onRoleChosen;
  final VoidCallback onRemoved;

  /// Shows the menu of [member] over the app. [onRoleChosen] and [onRemoved]
  /// run after the menu has closed.
  static Future<void> show(
    BuildContext context, {
    required MemberRow member,
    required ValueChanged<ShareRole> onRoleChosen,
    required VoidCallback onRemoved,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (context) => ShareMemberMenu(
        member: member,
        onRoleChosen: onRoleChosen,
        onRemoved: onRemoved,
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
        for (final role in shareableRoles)
          KansoSettingsRow(
            label: l10n.shareRoleName(role),
            trailing: role == member.role
                ? KansoIcon(KansoIcons.check, color: colors.accent)
                : null,
            onTap: () => choose(() => onRoleChosen(role)),
          ),
        const KansoDivider(),
        KansoSettingsRow(
          label: l10n.shareMemberRemove,
          danger: true,
          onTap: () => choose(onRemoved),
        ),
        const SizedBox(height: KansoSpacing.xs),
      ],
    );
  }
}
