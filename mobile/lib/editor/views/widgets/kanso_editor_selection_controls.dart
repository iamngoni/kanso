// Kanso — kanso_editor_selection_controls.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'kanso_selection_handle.dart';

/// The selection controls of the note's editor: handles Kanso draws, the
/// same on iOS and Android, and no toolbar of their own: the field's
/// `contextMenuBuilder` draws the only menu there is.
class KansoEditorSelectionControls extends TextSelectionControls
    with TextSelectionHandleControls {
  @override
  Size getHandleSize(double textLineHeight) =>
      KansoSelectionHandle.sizeFor(textLineHeight);

  @override
  Offset getHandleAnchor(TextSelectionHandleType type, double textLineHeight) =>
      KansoSelectionHandle.anchorFor(type, textLineHeight);

  @override
  Widget buildHandle(
    BuildContext context,
    TextSelectionHandleType type,
    double textLineHeight, [
    VoidCallback? onTap,
  ]) {
    return KansoSelectionHandle(type: type, lineHeight: textLineHeight);
  }
}
