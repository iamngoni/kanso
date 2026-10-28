// Kanso — note_export_progress.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// The part of the screen a sheet points at, from [context]'s box.
Rect? exportOriginOf(BuildContext context) {
  final box = context.findRenderObject();
  if (box is! RenderBox || !box.hasSize) {
    return null;
  }
  return box.localToGlobal(Offset.zero) & box.size;
}

/// What is shown while an export is written: [label] over a barrier that
/// lets nothing be tapped.
class NoteExportProgress extends StatelessWidget {
  const NoteExportProgress({required this.label, super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        const ModalBarrier(dismissible: false, color: Color(0x33000000)),
        Center(
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.lg,
              vertical: KansoSpacing.md,
            ),
            decoration: BoxDecoration(
              color: context.colors.elevated,
              borderRadius: KansoRadii.mediumBorder,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Text(label, style: context.typography.body),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
