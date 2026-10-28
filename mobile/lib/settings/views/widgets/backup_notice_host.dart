// Kanso — backup_notice_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../state/backup_notice/backup_notice_bloc.dart';
import 'backup_notice_card.dart';

/// [child] with the backup notice over it, on a veil, while the notice is to
/// be shown. Nothing below it can be touched until OK is pressed. It needs
/// a [BackupNoticeBloc] above it.
class BackupNoticeHost extends StatelessWidget {
  const BackupNoticeHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<BackupNoticeBloc, BackupNoticeState>(
      builder: (context, state) {
        return Stack(
          children: <Widget>[
            Positioned.fill(child: child),
            if (state.visible) ...<Widget>[
              const Positioned.fill(child: AbsorbPointer(child: KansoScrim())),
              Positioned.fill(
                // Above the app's navigator there is no page to give the
                // text its style, so the card sets plain text itself.
                child: KansoPlainText(
                  child: Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.lg),
                      child: BackupNoticeCard(
                        onOk: () => context.read<BackupNoticeBloc>().add(
                          const BackupNoticeEvent.dismissed(),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ],
        );
      },
    );
  }
}
