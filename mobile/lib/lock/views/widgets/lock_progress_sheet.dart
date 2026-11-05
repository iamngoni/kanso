// Kanso — lock_progress_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../note/views/widgets/attachment_sheet.dart' show byteSize;
import '../../utils/seal_progress_rules.dart';

/// The sheet a long lock shows: what it seals, a bar that counts the files
/// and, with [onCancel], a Cancel that stops it between two files.
///
/// [files] and [bytes] say how much there is before the first count arrives.
/// [unsealing] words it for taking a lock off.
class LockProgressSheet extends StatelessWidget {
  const LockProgressSheet({
    required this.title,
    required this.files,
    required this.bytes,
    this.progress,
    this.unsealing = false,
    this.cancelling = false,
    this.onCancel,
    super.key,
  });

  final String title;
  final int files;
  final int bytes;
  final SealProgress? progress;
  final bool unsealing;

  /// Whether Cancel was pressed and the lock is finishing the file it is on.
  final bool cancelling;
  final VoidCallback? onCancel;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final shown = progress;
    final total = shown?.total ?? files;
    final size = byteSize(shown?.bytesTotal ?? bytes);
    final line = unsealing
        ? l10n.lockProgressUnsealingLine(total, size)
        : l10n.lockProgressLine(total, size);
    return SingleChildScrollView(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(title, style: typography.heading2),
          const SizedBox(height: KansoSpacing.xs),
          Semantics(
            liveRegion: true,
            child: Text(line, style: typography.secondary),
          ),
          const SizedBox(height: KansoSpacing.sm),
          ClipRRect(
            borderRadius: BorderRadius.circular(3),
            child: LinearProgressIndicator(
              value: shown == null ? null : sealFraction(shown),
              minHeight: 6,
              color: colors.accent,
              backgroundColor: colors.fill,
            ),
          ),
          const SizedBox(height: KansoSpacing.xs),
          Text(
            shown == null
                ? ''
                : l10n.lockProgressCount(shown.done, shown.total),
            style: typography.secondary.copyWith(color: colors.textMuted),
          ),
          if (cancelling) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            Text(
              l10n.lockProgressStopping,
              style: typography.secondary.copyWith(color: colors.textMuted),
            ),
          ],
          if (onCancel != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: l10n.lockProgressCancel,
              variant: KansoButtonVariant.quiet,
              onPressed: cancelling ? null : onCancel,
            ),
          ],
        ],
      ),
    );
  }
}
