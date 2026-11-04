// Kanso — canvas_banner.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../state/canvas/canvas_bloc.dart';

/// A quiet line above the canvas when it is near its size limit, full, or
/// could not be written. Nothing at all (no space) when there is nothing
/// to say.
class CanvasBanner extends StatelessWidget {
  const CanvasBanner({required this.view, required this.problem, super.key});

  final CanvasView view;
  final CanvasProblem? problem;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final String? message;
    var tone = KansoMessageTone.info;
    if (problem == CanvasProblem.notSaved) {
      message = l10n.canvasNotSaved;
      tone = KansoMessageTone.error;
    } else if (view.budget == CanvasBudget.full ||
        problem == CanvasProblem.full) {
      message =
          '${l10n.canvasBudgetFullTitle}. ${l10n.canvasBudgetFullMessage}';
      tone = KansoMessageTone.error;
    } else if (view.budget == CanvasBudget.warn) {
      message =
          '${l10n.canvasBudgetWarnTitle}. ${l10n.canvasBudgetWarnMessage}';
    } else {
      message = null;
    }
    if (message == null) {
      return const SizedBox.shrink();
    }
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        0,
      ),
      child: KansoInlineMessage(message: message, tone: tone),
    );
  }
}
