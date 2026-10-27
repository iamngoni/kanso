// Kanso — kanso_text_context_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../utils/context_extensions.dart';
import 'kanso_text_menu_toolbar.dart';

/// The menu over selected or tapped text in a field: Cut, Copy, Paste and
/// Select All, in the buttons the field offers for what is selected, drawn
/// by Kanso so it is the same on iOS and Android.
///
/// It is made for `EditableText.contextMenuBuilder`: it sits above the
/// selection, or below it where there is no room above.
class KansoTextContextMenu extends StatelessWidget {
  const KansoTextContextMenu({
    required this.state,
    this.extra = const <(String, VoidCallback)>[],
    super.key,
  });

  final EditableTextState state;

  /// Buttons of the screen's own, before the field's (Open link).
  final List<(String, VoidCallback)> extra;

  /// Whether there is text and not all of it is selected. The platform's own
  /// rule hides Select All on iOS over a selection; this one does not.
  static bool everythingNotSelected(EditableTextState state) {
    final value = state.textEditingValue;
    if (value.text.isEmpty) {
      return false;
    }
    final selection = value.selection;
    return !(selection.isValid &&
        selection.start == 0 &&
        selection.end == value.text.length);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    // The platform decides which buttons its own menu has (iOS has no
    // Select All over a selection); Kanso's has the same on both.
    final buttons = <(String, VoidCallback)>[
      ...extra,
      if (state.cutEnabled)
        (l10n.menuCut, () => state.cutSelection(SelectionChangedCause.toolbar)),
      if (state.copyEnabled)
        (
          l10n.menuCopy,
          () => state.copySelection(SelectionChangedCause.toolbar),
        ),
      if (state.pasteEnabled)
        (l10n.menuPaste, () => state.pasteText(SelectionChangedCause.toolbar)),
      if (everythingNotSelected(state))
        (
          l10n.menuSelectAll,
          () => state.selectAll(SelectionChangedCause.toolbar),
        ),
    ];
    if (buttons.isEmpty) {
      return const SizedBox.shrink();
    }
    return KansoTextMenuToolbar(
      anchors: KansoTextMenuToolbar.anchorsOf(state),
      buttons: buttons,
    );
  }
}
