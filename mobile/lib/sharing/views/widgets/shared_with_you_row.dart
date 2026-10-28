// Kanso — shared_with_you_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../state/invitations/invitations_bloc.dart';

/// The library's row for the notes shared with this account: "Shared with
/// you", and a badge with how many invitations wait when there are any.
/// Pressing it reads the invitations again and runs [onOpen], which opens
/// the screen. It reads the app's `InvitationsBloc`.
class SharedWithYouRow extends StatelessWidget {
  const SharedWithYouRow({
    required this.onOpen,
    this.selected = false,
    super.key,
  });

  final VoidCallback onOpen;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    return BlocSelector<InvitationsBloc, InvitationsState, int>(
      selector: (state) => state.invitations.length,
      builder: (context, waiting) {
        return KansoSidebarRow(
          icon: KansoIcons.users,
          label: context.l10n.sharedWithYouTitle,
          selected: selected,
          badge: waiting == 0
              ? null
              : KansoBadge(label: '$waiting', tone: KansoBadgeTone.accent),
          onTap: () {
            context.read<InvitationsBloc>().add(
              const InvitationsEvent.refreshed(),
            );
            onOpen();
          },
        );
      },
    );
  }
}
