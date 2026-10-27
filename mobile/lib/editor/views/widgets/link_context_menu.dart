// Kanso — link_context_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_text_context_menu.dart';
import '../../models/repos/abs/editor_repository.dart';

/// The editor's text menu: Cut, Copy, Paste and Select All, and, where the
/// caret is in a link, Open link first.
///
/// Where a link leads is the Rust core's to say (`EditorDocument.linkAt`); the
/// menu opens at once and gains the button when the answer comes.
class LinkContextMenu extends StatefulWidget {
  const LinkContextMenu({
    required this.state,
    required this.document,
    required this.onOpen,
    this.windowStart = 0,
    super.key,
  });

  final EditableTextState state;
  final EditorDocument document;

  /// Where the field's text starts in the note's, when the field holds only
  /// a window of a long note.
  final int windowStart;

  /// The person chose Open link, for a link that leads where this says.
  final ValueChanged<LinkTarget> onOpen;

  @override
  State<LinkContextMenu> createState() => _LinkContextMenuState();
}

class _LinkContextMenuState extends State<LinkContextMenu> {
  late final Future<LinkTarget?> target;

  @override
  void initState() {
    super.initState();
    final at = widget.state.textEditingValue.selection;
    target = at.isValid
        ? widget.document
              .linkAt(widget.windowStart + at.start)
              .then(
                (found) => found.fold<LinkTarget?>((_) => null, (hit) => hit),
              )
        : Future<LinkTarget?>.value();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<LinkTarget?>(
      future: target,
      builder: (context, snapshot) {
        final found = snapshot.data;
        return KansoTextContextMenu(
          state: widget.state,
          extra: <(String, VoidCallback)>[
            if (found != null)
              (
                context.l10n.menuOpenLink,
                () {
                  widget.state.hideToolbar();
                  widget.onOpen(found);
                },
              ),
          ],
        );
      },
    );
  }
}
