// Kanso — share_invite_bar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../models/data/share_done.dart';
import '../../state/share_note/share_note_bloc.dart';
import 'share_role_segmented.dart';

/// The field for the username of the person to invite, the role they get and
/// the Invite button. It needs a [ShareNoteBloc] above it.
class ShareInviteBar extends StatefulWidget {
  const ShareInviteBar({super.key});

  @override
  State<ShareInviteBar> createState() => _ShareInviteBarState();
}

class _ShareInviteBarState extends State<ShareInviteBar> {
  late final TextEditingController username;

  @override
  void initState() {
    super.initState();
    username = TextEditingController();
  }

  @override
  void dispose() {
    username.dispose();
    super.dispose();
  }

  void invite(BuildContext context) {
    context.read<ShareNoteBloc>().add(ShareNoteEvent.invited(username.text));
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<ShareNoteBloc, ShareNoteState>(
      listenWhen: (before, after) =>
          before.done != after.done && after.done is ShareDoneInvited,
      // The person is invited: the field is ready for the next one.
      listener: (context, state) => username.clear(),
      buildWhen: (before, after) =>
          before.busy != after.busy || before.inviteRole != after.inviteRole,
      builder: (context, state) {
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Row(
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
                KansoButton(
                  label: l10n.shareInvite,
                  expanded: false,
                  onPressed: state.busy ? null : () => invite(context),
                ),
              ],
            ),
            const SizedBox(height: KansoSpacing.xs),
            ShareRoleSegmented(
              selected: state.inviteRole,
              semanticLabel: l10n.shareInviteRoleLabel,
              onChanged: (role) => context.read<ShareNoteBloc>().add(
                ShareNoteEvent.inviteRoleChosen(role),
              ),
            ),
          ],
        );
      },
    );
  }
}
