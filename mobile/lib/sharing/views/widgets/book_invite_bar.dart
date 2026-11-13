// Kanso — book_invite_bar.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../models/data/book_prompt.dart';
import '../../state/share_notebook/share_notebook_bloc.dart';
import '../../utils/book_labels.dart';

/// The desktop's "People: username, Edit ▾, Invite": the username of
/// the person to invite, the role they get (only the roles the Rust core
/// lets this account give) and the button, which shares the notebook when
/// it isn't shared yet. It needs a [ShareNotebookBloc] above it.
class BookInviteBar extends StatefulWidget {
  const BookInviteBar({required this.roles, super.key});

  /// The roles this account may invite with, highest first.
  final List<BookRole> roles;

  @override
  State<BookInviteBar> createState() => _BookInviteBarState();
}

class _BookInviteBarState extends State<BookInviteBar> {
  late final TextEditingController username = TextEditingController();

  @override
  void dispose() {
    username.dispose();
    super.dispose();
  }

  void invite(BuildContext context) {
    context.read<ShareNotebookBloc>().add(
      ShareNotebookEvent.invited(username.text),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<ShareNotebookBloc, ShareNotebookState>(
      listenWhen: (before, after) =>
          before.done != after.done &&
          (after.done is BookDoneInvited || after.done is BookDoneShared),
      // The person is invited: the field is ready for the next one.
      listener: (context, state) => username.clear(),
      buildWhen: (before, after) =>
          before.busy != after.busy ||
          before.inviteRole != after.inviteRole ||
          (before.info == null) != (after.info == null),
      builder: (context, state) {
        final first = state.info == null;
        return Row(
          children: <Widget>[
            Expanded(
              child: KansoTextInput(
                controller: username,
                hint: l10n.shareUsernameHint,
                autocorrect: false,
                enabled: !state.busy,
                textInputAction: TextInputAction.send,
                onSubmitted: (_) => invite(context),
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            _RolePicker(
              role: state.inviteRole,
              roles: widget.roles,
              enabled: !state.busy,
              onChosen: (role) => context.read<ShareNotebookBloc>().add(
                ShareNotebookEvent.inviteRoleChosen(role),
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
            KansoButton(
              label: first ? l10n.shareNotebookShare : l10n.shareInvite,
              expanded: false,
              onPressed: state.busy ? null : () => invite(context),
            ),
          ],
        );
      },
    );
  }
}

/// The role the next person gets, as a quiet pill with a chevron that opens
/// the roles to choose from.
class _RolePicker extends StatelessWidget {
  const _RolePicker({
    required this.role,
    required this.roles,
    required this.enabled,
    required this.onChosen,
  });

  final BookRole role;
  final List<BookRole> roles;
  final bool enabled;
  final ValueChanged<BookRole> onChosen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return KansoPressable(
      semanticLabel:
          '${l10n.shareNotebookInviteRoleLabel}: ${bookRoleLabel(l10n, role)}',
      onTap: enabled && roles.length > 1
          ? () => showKansoBottomSheet<void>(
              context,
              builder: (sheet) => Column(
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
                      l10n.shareNotebookInviteRoleLabel,
                      style: sheet.typography.heading2,
                    ),
                  ),
                  for (final option in roles)
                    KansoSettingsRow(
                      label: bookRoleLabel(l10n, option),
                      trailing: option == role
                          ? KansoIcon(KansoIcons.check, color: colors.accent)
                          : null,
                      onTap: () {
                        Navigator.of(sheet).pop();
                        onChosen(option);
                      },
                    ),
                  const SizedBox(height: KansoSpacing.xs),
                ],
              ),
            )
          : null,
      child: ExcludeSemantics(
        child: Container(
          height: 40,
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
          decoration: BoxDecoration(
            color: colors.fill,
            borderRadius: KansoRadii.pillBorder,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text(
                bookRoleLabel(l10n, role),
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
              const SizedBox(width: KansoSpacing.xxs),
              KansoIcon(
                KansoIcons.chevronDown,
                size: 14,
                color: colors.textMuted,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
