// Kanso — canvas_toolbar.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../models/data/canvas_tool.dart';
import '../../utils/ink_colors.dart';

/// The tools along the foot of the canvas, and under them what the tool in
/// hand needs: colors and a width for a pen, or what to do with a selection.
class CanvasToolbar extends StatelessWidget {
  const CanvasToolbar({
    required this.tool,
    required this.color,
    required this.width,
    required this.dark,
    required this.text,
    required this.selected,
    required this.onTool,
    required this.onColor,
    required this.onWidth,
    required this.onDeleteSelection,
    required this.onClearSelection,
    this.onMoveToNew,
    this.toolList = tools,
    super.key,
  });

  final CanvasTool tool;
  final String color;
  final double width;
  final bool dark;
  final Color text;

  /// The strokes the lasso holds.
  final List<StrokeKey> selected;
  final ValueChanged<CanvasTool> onTool;
  final ValueChanged<String> onColor;
  final ValueChanged<double> onWidth;
  final VoidCallback onDeleteSelection;
  final VoidCallback onClearSelection;

  /// Moves a selection to a new canvas; null hides the button (a sketch
  /// in a note has none).
  final VoidCallback? onMoveToNew;

  /// The tools offered, in order; a sketch offers only the ink ones.
  final List<CanvasTool> toolList;

  /// The tools in the order they are laid out.
  static const List<CanvasTool> tools = <CanvasTool>[
    CanvasTool.pen,
    CanvasTool.pencil,
    CanvasTool.marker,
    CanvasTool.highlighter,
    CanvasTool.eraser,
    CanvasTool.partialEraser,
    CanvasTool.lasso,
    CanvasTool.text,
    CanvasTool.arrow,
    CanvasTool.hand,
  ];

  /// The words for a tool, for its label.
  static String label(AppLocalizations l10n, CanvasTool tool) => switch (tool) {
    CanvasTool.pen => l10n.canvasToolPen,
    CanvasTool.pencil => l10n.canvasToolPencil,
    CanvasTool.marker => l10n.canvasToolMarker,
    CanvasTool.highlighter => l10n.canvasToolHighlighter,
    CanvasTool.eraser => l10n.canvasToolEraser,
    CanvasTool.partialEraser => l10n.canvasToolPartialEraser,
    CanvasTool.lasso => l10n.canvasToolLasso,
    CanvasTool.text => l10n.canvasToolText,
    CanvasTool.arrow => l10n.canvasToolArrow,
    CanvasTool.hand => l10n.canvasToolHand,
  };

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.elevated,
        border: Border(top: BorderSide(color: colors.divider)),
      ),
      child: SafeArea(
        top: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            if (tool.draws)
              _inkRow(context)
            else if (selected.isNotEmpty)
              _selectionRow(context),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.symmetric(
                horizontal: KansoSpacing.xs,
                vertical: KansoSpacing.xxs,
              ),
              child: Row(
                children: <Widget>[
                  for (final t in toolList)
                    _ToolButton(
                      tool: t,
                      selected: t == tool,
                      label: label(l10n, t),
                      onTap: () => onTool(t),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _inkRow(BuildContext context) {
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        0,
      ),
      child: Row(
        children: <Widget>[
          for (final hex in inkPalette)
            Semantics(
              button: true,
              selected: hex == color,
              label: '${context.l10n.canvasColor} $hex',
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => onColor(hex),
                child: Padding(
                  padding: const EdgeInsets.only(right: KansoSpacing.sm),
                  child: Container(
                    width: 28,
                    height: 28,
                    decoration: BoxDecoration(
                      color: inkColor(hex, dark: dark, text: text),
                      shape: BoxShape.circle,
                      border: Border.all(
                        color: hex == color ? colors.accent : colors.divider,
                        width: hex == color ? 3 : 1,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          Expanded(
            child: Semantics(
              label: context.l10n.canvasWidth,
              child: Material(
                type: MaterialType.transparency,
                child: Slider(
                  value: width.clamp(1, 24).toDouble(),
                  min: 1,
                  max: 24,
                  onChanged: onWidth,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _selectionRow(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        0,
      ),
      child: Row(
        children: <Widget>[
          if (onMoveToNew != null) ...<Widget>[
            Expanded(
              child: KansoButton(
                label: l10n.canvasMoveToNew,
                icon: KansoIcons.arrowRight,
                variant: KansoButtonVariant.secondary,
                compact: true,
                onPressed: onMoveToNew,
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
          ] else
            const Spacer(),
          KansoButton(
            label: l10n.canvasSelectionDelete,
            icon: KansoIcons.trash,
            variant: KansoButtonVariant.danger,
            compact: true,
            expanded: false,
            onPressed: onDeleteSelection,
          ),
          const SizedBox(width: KansoSpacing.xs),
          KansoButton(
            label: l10n.canvasSelectionClear,
            variant: KansoButtonVariant.quiet,
            compact: true,
            expanded: false,
            onPressed: onClearSelection,
          ),
        ],
      ),
    );
  }
}

class _ToolButton extends StatelessWidget {
  const _ToolButton({
    required this.tool,
    required this.selected,
    required this.label,
    required this.onTap,
  });

  final CanvasTool tool;
  final bool selected;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      button: true,
      selected: selected,
      label: label,
      child: KansoPressable(
        onTap: onTap,
        child: Container(
          width: 44,
          height: 44,
          margin: const EdgeInsets.symmetric(horizontal: 2),
          decoration: BoxDecoration(
            color: selected ? colors.selected : null,
            borderRadius: BorderRadius.circular(10),
          ),
          alignment: Alignment.center,
          child: KansoIcon(
            tool.icon,
            size: 22,
            color: selected ? colors.accent : colors.textSecondary,
          ),
        ),
      ),
    );
  }
}
