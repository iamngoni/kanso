// Kanso — kanso_text_menu_toolbar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';
import 'kanso_pressable.dart';

/// A row of text buttons over the text of a field, as Kanso draws the menus
/// of its fields: a floating card with a hairline between the buttons, above
/// the anchor or below it where there is no room above. The same on iOS and
/// Android.
class KansoTextMenuToolbar extends StatelessWidget {
  const KansoTextMenuToolbar({
    required this.anchors,
    required this.buttons,
    super.key,
  });

  final TextSelectionToolbarAnchors anchors;

  /// The label and action of each button, in order.
  final List<(String, VoidCallback)> buttons;

  /// Where a menu over [state]'s selection is anchored: above its first line
  /// and below its last, worked out the same on every platform (the
  /// platforms' own differ).
  static TextSelectionToolbarAnchors anchorsOf(EditableTextState state) {
    final render = state.renderEditable;
    final points = render.getEndpointsForSelection(
      state.textEditingValue.selection,
    );
    if (points.isEmpty) {
      return state.contextMenuAnchors;
    }
    final line = render.preferredLineHeight;
    return TextSelectionToolbarAnchors.fromSelection(
      renderBox: render,
      startGlyphHeight: line,
      endGlyphHeight: line,
      selectionEndpoints: points,
    );
  }

  /// The most the menu is wide, in points.
  static const double maxWidth = 320;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    // The anchors are on the screen; the layout starts inside the padding.
    final inset = MediaQuery.paddingOf(context).top;
    final shift = Offset(inset, inset);
    return Padding(
      padding: EdgeInsets.all(inset),
      child: CustomSingleChildLayout(
        delegate: TextSelectionToolbarLayoutDelegate(
          anchorAbove:
              anchors.primaryAnchor - const Offset(0, KansoSpacing.xs) - shift,
          anchorBelow:
              (anchors.secondaryAnchor ?? anchors.primaryAnchor) +
              const Offset(0, KansoSpacing.xs) -
              shift,
        ),
        child: Container(
          constraints: const BoxConstraints(maxWidth: maxWidth),
          decoration: BoxDecoration(
            color: colors.elevated,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(color: colors.divider),
            boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
          ),
          child: ClipRRect(
            borderRadius: KansoRadii.mediumBorder,
            child: IntrinsicHeight(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  for (var i = 0; i < buttons.length; i++) ...<Widget>[
                    if (i > 0) const KansoDivider(vertical: true),
                    KansoPressable(
                      onTap: buttons[i].$2,
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: KansoSpacing.md,
                          vertical: KansoSpacing.sm,
                        ),
                        child: Text(
                          buttons[i].$1,
                          style: context.typography.body,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
