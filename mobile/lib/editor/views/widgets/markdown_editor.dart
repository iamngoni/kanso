// Kanso — markdown_editor.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_text_context_menu.dart';
import '../controllers/kanso_markdown_controller.dart';
import 'kanso_editable_text.dart';
import 'kanso_editor_selection_controls.dart';
import 'kanso_magnifier.dart';
import 'kanso_spell_check_menu.dart';
import 'kanso_undo_manager_client.dart';

/// The note's text as Markdown to edit: the text of the
/// [KansoMarkdownController] in the editor font, coloured by the theme,
/// with Kanso's caret, handles, magnifier and menus, the same on iOS and
/// Android.
///
/// It lays out all its text and does not scroll: it goes in a scroll view of
/// the screen's. A tap places the caret (it never follows a link); a long
/// press or a double tap selects. Undo and redo, from the keyboard or the
/// system, are the Rust core's, not Flutter's.
class MarkdownEditor extends StatefulWidget {
  const MarkdownEditor({
    required this.controller,
    required this.focusNode,
    this.autofocus = false,
    this.spellCheckService,
    this.scrollPadding = const EdgeInsets.all(24),
    this.menuBuilder,
    this.editableKey,
    super.key,
  });

  final KansoMarkdownController controller;
  final FocusNode focusNode;
  final bool autofocus;

  /// Checks the spelling; none turns spell check off.
  final SpellCheckService? spellCheckService;

  /// How much room to keep around the caret when it is scrolled into view.
  final EdgeInsets scrollPadding;

  /// The menu over the text, in place of the plain one (Cut, Copy, Paste and
  /// Select All): the screen's own, with Open link when the caret is in a
  /// link.
  final EditableTextContextMenuBuilder? menuBuilder;

  /// A key for the editable text inside, to find where the caret is.
  final GlobalKey<EditableTextState>? editableKey;

  /// How tall the caret is, in points, for an editor font [fontSize] high.
  static double caretHeightFor(double fontSize) =>
      (fontSize * 1.25).roundToDouble();

  @override
  State<MarkdownEditor> createState() => _MarkdownEditorState();
}

class _MarkdownEditorState extends State<MarkdownEditor>
    implements TextSelectionGestureDetectorBuilderDelegate {
  late final TextSelectionGestureDetectorBuilder gestures;
  final KansoEditorSelectionControls selectionControls =
      KansoEditorSelectionControls();
  late KansoUndoManagerClient undoClient;

  final GlobalKey<EditableTextState> ownKey = GlobalKey<EditableTextState>();

  @override
  GlobalKey<EditableTextState> get editableTextKey =>
      widget.editableKey ?? ownKey;

  @override
  bool get forcePressEnabled => false;

  @override
  bool get selectionEnabled => true;

  @override
  void initState() {
    super.initState();
    gestures = TextSelectionGestureDetectorBuilder(delegate: this);
    undoClient = KansoUndoManagerClient(
      controller: widget.controller,
      focusNode: widget.focusNode,
    )..attach();
  }

  @override
  void didUpdateWidget(MarkdownEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != widget.controller ||
        oldWidget.focusNode != widget.focusNode) {
      undoClient.detach();
      undoClient = KansoUndoManagerClient(
        controller: widget.controller,
        focusNode: widget.focusNode,
      )..attach();
    }
  }

  @override
  void dispose() {
    undoClient.detach();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final style = context.typography.editorBody.copyWith(
      color: context.syntaxColors.text,
    );
    final apple = switch (Theme.of(context).platform) {
      TargetPlatform.iOS || TargetPlatform.macOS => true,
      _ => false,
    };
    // The framework draws the caret 2 points taller on Apple's platforms:
    // ask for less there, so the two carets match.
    final caret = MarkdownEditor.caretHeightFor(style.fontSize ?? 16);
    final service = widget.spellCheckService;
    widget.controller.misspelledStyle = widget.controller.misspelledStyle
        .copyWith(decorationColor: colors.danger);
    return Actions(
      actions: <Type, Action<Intent>>{
        UndoTextIntent: CallbackAction<UndoTextIntent>(
          onInvoke: (_) => widget.controller.undo(),
        ),
        RedoTextIntent: CallbackAction<RedoTextIntent>(
          onInvoke: (_) => widget.controller.redo(),
        ),
      },
      child: gestures.buildGestureDetector(
        behavior: HitTestBehavior.translucent,
        child: KansoEditableText(
          key: editableTextKey,
          controller: widget.controller,
          focusNode: widget.focusNode,
          autofocus: widget.autofocus,
          style: style,
          cursorColor: colors.accent,
          backgroundCursorColor: colors.textMuted,
          selectionColor: colors.accent.withValues(alpha: 0.3),
          cursorRadius: const Radius.circular(1),
          cursorHeight: apple ? caret - 2 : caret,
          scrollPadding: widget.scrollPadding,
          keyboardAppearance: Theme.brightnessOf(context),
          selectionControls: selectionControls,
          magnifierConfiguration: KansoMagnifier.configuration(),
          contextMenuBuilder:
              widget.menuBuilder ??
              (context, state) => KansoTextContextMenu(state: state),
          spellCheckConfiguration: service == null
              ? const SpellCheckConfiguration.disabled()
              : SpellCheckConfiguration(
                  spellCheckService: service,
                  misspelledTextStyle: widget.controller.misspelledStyle,
                  spellCheckSuggestionsToolbarBuilder: (context, state) =>
                      KansoSpellCheckMenu(state: state),
                ),
        ),
      ),
    );
  }
}
