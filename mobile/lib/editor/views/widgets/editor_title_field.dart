// Kanso — editor_title_field.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_selection_controls.dart';
import '../../../core/views/widgets/kanso_text_context_menu.dart';
import 'markdown_editor.dart';

/// The title of the note being edited: large, wrapping, with the hint
/// "Title" while it is empty. Return moves on to the text.
///
/// Like the search field it is Flutter's `EditableText` with Kanso's caret
/// and menu, and no handles or magnifier, so iOS and Android draw it alike.
class EditorTitleField extends StatefulWidget {
  const EditorTitleField({
    required this.controller,
    required this.focusNode,
    required this.onChanged,
    required this.onSubmitted,
    super.key,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final ValueChanged<String> onChanged;

  /// Return was pressed: go on to the text.
  final VoidCallback onSubmitted;

  @override
  State<EditorTitleField> createState() => _EditorTitleFieldState();
}

class _EditorTitleFieldState extends State<EditorTitleField>
    implements TextSelectionGestureDetectorBuilderDelegate {
  late final TextSelectionGestureDetectorBuilder gestures;
  final TextSelectionControls selectionControls = KansoSelectionControls();

  @override
  final GlobalKey<EditableTextState> editableTextKey =
      GlobalKey<EditableTextState>();

  @override
  bool get forcePressEnabled => false;

  @override
  bool get selectionEnabled => true;

  @override
  void initState() {
    super.initState();
    gestures = TextSelectionGestureDetectorBuilder(delegate: this);
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final style = context.typography.editorTitle;
    final apple = switch (Theme.of(context).platform) {
      TargetPlatform.iOS || TargetPlatform.macOS => true,
      _ => false,
    };
    final caret = MarkdownEditor.caretHeightFor(style.fontSize ?? 24);
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return Stack(
          children: <Widget>[
            if (widget.controller.text.isEmpty)
              IgnorePointer(
                child: Text(
                  context.l10n.editorTitleHint,
                  style: style.copyWith(color: colors.textMuted),
                ),
              ),
            gestures.buildGestureDetector(
              behavior: HitTestBehavior.translucent,
              child: EditableText(
                key: editableTextKey,
                rendererIgnoresPointer: true,
                controller: widget.controller,
                focusNode: widget.focusNode,
                style: style,
                cursorColor: colors.accent,
                backgroundCursorColor: colors.textMuted,
                selectionColor: colors.accent.withValues(alpha: 0.3),
                cursorRadius: const Radius.circular(1),
                cursorHeight: apple ? caret - 2 : caret,
                maxLines: null,
                keyboardType: TextInputType.text,
                textInputAction: TextInputAction.next,
                textCapitalization: TextCapitalization.sentences,
                keyboardAppearance: Theme.brightnessOf(context),
                selectionControls: selectionControls,
                contextMenuBuilder: (context, state) =>
                    KansoTextContextMenu(state: state),
                onChanged: widget.onChanged,
                onEditingComplete: widget.onSubmitted,
              ),
            ),
          ],
        );
      },
    );
  }
}
