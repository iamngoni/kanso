// Kanso — locked_note_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../lock/models/data/lock_target.dart';
import '../../../lock/models/data/unlock_status.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../lock/state/unlock/unlock_bloc.dart';
import '../../../lock/utils/auto_lock.dart';
import '../../../lock/utils/biometric_kinds.dart';
import '../../../lock/views/widgets/unlock_sheet.dart';

/// A locked note: its title (titles are not sealed) with its lock, and in
/// place of the text a calm card that offers to unlock it: with Face ID,
/// Touch ID or the fingerprint when they are on, and always with the lock
/// password.
///
/// Nothing else of the note is shown (not its facts, and not its text).
/// It reads the `LockStatusBloc` and the `UnlockBloc` above it.
class LockedNotePane extends StatelessWidget {
  const LockedNotePane({
    required this.noteId,
    required this.title,
    required this.gutter,
    super.key,
  });

  final String noteId;
  final String title;

  /// The space either side of the title.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: EdgeInsets.fromLTRB(gutter, KansoSpacing.xxs, gutter, 0),
          child: Row(
            children: <Widget>[
              Flexible(
                child: Text(
                  title.trim().isEmpty ? l10n.noteUntitled : title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: context.typography.previewTitle,
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              KansoIcon(KansoIcons.lock, color: colors.textMuted),
            ],
          ),
        ),
        Expanded(
          // The card is centred, and scrolls when a large device text size
          // makes it taller than the room.
          child: LayoutBuilder(
            builder: (context, box) => SingleChildScrollView(
              padding: EdgeInsets.all(gutter),
              child: ConstrainedBox(
                constraints: BoxConstraints(
                  minHeight: box.maxHeight - 2 * gutter,
                ),
                child: Center(child: buildCard(context)),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget buildCard(BuildContext context) {
    final l10n = context.l10n;
    return BlocListener<UnlockBloc, UnlockState>(
      listenWhen: (before, after) =>
          before.status != after.status &&
          after.status == UnlockStatus.invalidated,
      // The biometrics changed: Rust forgot the keys, so they are off now.
      listener: (context, state) =>
          context.read<LockStatusBloc>().add(const LockStatusEvent.refreshed()),
      child: BlocBuilder<LockStatusBloc, LockStatusState>(
        builder: (context, lock) {
          return BlocBuilder<UnlockBloc, UnlockState>(
            builder: (context, unlock) {
              final method = biometricName(l10n, lock.kind);
              final working = unlock.status == UnlockStatus.working;
              return KansoMessageCard(
                icon: KansoIcons.lock,
                title: l10n.lockedTitle,
                message: switch (unlock.status) {
                  UnlockStatus.invalidated => l10n.lockedInvalidated(method),
                  UnlockStatus.failed => l10n.lockedBiometricFailed(method),
                  _ => l10n.lockedMessage(autoLockMinutes),
                },
                actions: <Widget>[
                  if (lock.biometricsUsable)
                    KansoButton(
                      label: l10n.lockedUnlockWith(method),
                      icon: switch (lock.kind) {
                        BiometricKind.faceId ||
                        BiometricKind.face => KansoIcons.scanFace,
                        _ => KansoIcons.fingerprint,
                      },
                      onPressed: working
                          ? null
                          : () => context.read<UnlockBloc>().add(
                              UnlockEvent.biometricRequested(
                                l10n.biometricReasonUnlock,
                              ),
                            ),
                    ),
                  KansoButton(
                    label: l10n.lockedUsePassword,
                    variant: lock.biometricsUsable
                        ? KansoButtonVariant.secondary
                        : KansoButtonVariant.primary,
                    onPressed: working
                        ? null
                        : () => UnlockSheet.show(
                            context,
                            target: LockTarget.note(noteId),
                            bloc: context.read<UnlockBloc>(),
                            status: context.read<LockStatusBloc>(),
                          ),
                  ),
                ],
              );
            },
          );
        },
      ),
    );
  }
}
