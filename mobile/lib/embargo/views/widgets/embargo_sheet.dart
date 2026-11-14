// Kanso — embargo_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/embargo_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../injection.dart';
import '../../models/data/embargo_gate.dart';
import '../../models/repos/abs/embargo_repository.dart';

/// Asks before [gate] when note [noteId] is under embargo, and answers
/// whether to go ahead: at once when it has none (or the phone can't tell),
/// and otherwise only when the person says to go ahead anyway.
///
/// [repository] is the app's when null (tests pass one).
Future<bool> confirmEmbargo(
  BuildContext context, {
  required String noteId,
  required EmbargoGate gate,
  EmbargoRepository? repository,
}) async {
  final repo =
      repository ??
      (getIt.isRegistered<EmbargoRepository>()
          ? getIt<EmbargoRepository>()
          : null);
  if (repo == null) {
    return true;
  }
  final read = await repo.embargo(noteId);
  final embargo = read.toOption().toNullable();
  if (embargo == null || !context.mounted) {
    return embargo == null;
  }
  final answer = await showKansoBottomSheet<bool>(
    context,
    builder: (_) => EmbargoSheet(embargo: embargo, gate: gate),
  );
  return answer ?? false;
}

/// The question before going past an embargo: a pause mark, "This note is
/// under embargo", when it lifts and how far off that is, and the button
/// that goes ahead anyway over the one that keeps it back.
class EmbargoSheet extends StatelessWidget {
  const EmbargoSheet({required this.embargo, required this.gate, super.key});

  final EmbargoView embargo;
  final EmbargoGate gate;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final (question, confirm) = switch (gate) {
      EmbargoGate.publish => (
        l10n.embargoAskPublish(embargo.lifts, embargo.liftsIn),
        l10n.embargoPublishAnyway,
      ),
      EmbargoGate.shareLink => (
        l10n.embargoAskShareLink(embargo.lifts, embargo.liftsIn),
        l10n.embargoShareLinkAnyway,
      ),
      EmbargoGate.export => (
        l10n.embargoAskExport(embargo.lifts, embargo.liftsIn),
        l10n.embargoExportAnyway,
      ),
    };
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: Container(
              width: 44,
              height: 44,
              decoration: BoxDecoration(
                color: colors.warning.withValues(alpha: 0.16),
                shape: BoxShape.circle,
              ),
              alignment: Alignment.center,
              child: KansoIcon(
                KansoIcons.pause,
                size: 22,
                color: colors.warning,
              ),
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
          Text(l10n.embargoAskTitle, style: context.typography.heading2),
          const SizedBox(height: KansoSpacing.xs),
          Text(question, style: context.typography.body),
          const SizedBox(height: KansoSpacing.sm),
          Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            decoration: BoxDecoration(
              color: colors.fill,
              borderRadius: KansoRadii.mediumBorder,
            ),
            child: Text(
              l10n.embargoGuardRail,
              style: context.typography.meta.copyWith(
                color: colors.textSecondary,
              ),
            ),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: confirm,
            onPressed: () => Navigator.of(context).pop(true),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.cancel,
            variant: KansoButtonVariant.secondary,
            onPressed: () => Navigator.of(context).pop(false),
          ),
        ],
      ),
    );
  }
}
