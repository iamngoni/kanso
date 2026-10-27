// Kanso — kanso_text_input.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A line of text a person types in, over Flutter's `EditableText` with
/// Kanso's caret and menu, and no handles or magnifier, so iOS and Android
/// draw it alike: a filled field with the accent outline while it has focus.
///
/// The caller owns the [controller] and [focusNode]. [hint] shows while the
/// field is empty; [semanticLabel] says what the field is for.
class KansoTextInput extends StatefulWidget {
  const KansoTextInput({
    required this.controller,
    required this.focusNode,
    this.hint,
    this.semanticLabel,
    this.keyboardType = TextInputType.text,
    this.textInputAction = TextInputAction.done,
    this.textCapitalization = TextCapitalization.none,
    this.autocorrect = true,
    this.onChanged,
    this.onSubmitted,
    super.key,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final String? hint;
  final String? semanticLabel;
  final TextInputType keyboardType;
  final TextInputAction textInputAction;
  final TextCapitalization textCapitalization;
  final bool autocorrect;
  final ValueChanged<String>? onChanged;

  /// The return key was pressed, with the text as it is.
  final ValueChanged<String>? onSubmitted;

  /// How tall the caret is, in points.
  static const double caretHeight = 18;

  @override
  State<KansoTextInput> createState() => _KansoTextInputState();
}

class _KansoTextInputState extends State<KansoTextInput>
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
    final typography = context.typography;
    // The caret the framework draws is taller on Apple's platforms: ask for
    // less there, so both match.
    final asked = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => KansoTextInput.caretHeight - 2,
      _ => KansoTextInput.caretHeight,
    };
    return ListenableBuilder(
      listenable: Listenable.merge(<Listenable>[
        widget.controller,
        widget.focusNode,
      ]),
      builder: (context, _) {
        final focused = widget.focusNode.hasFocus;
        final hint = widget.hint;
        return Semantics(
          label: widget.semanticLabel,
          textField: true,
          child: Container(
            height: KansoSizes.searchField,
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
            alignment: AlignmentDirectional.centerStart,
            decoration: BoxDecoration(
              color: colors.fill,
              borderRadius: KansoRadii.mediumBorder,
              border: Border.all(
                color: focused ? colors.accent : colors.fill,
                width: focused ? 1.5 : 1,
              ),
            ),
            child: Stack(
              alignment: AlignmentDirectional.centerStart,
              children: <Widget>[
                if (hint != null && widget.controller.text.isEmpty)
                  IgnorePointer(
                    child: Text(
                      hint,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.body.copyWith(color: colors.textMuted),
                    ),
                  ),
                gestures.buildGestureDetector(
                  behavior: HitTestBehavior.translucent,
                  child: EditableText(
                    key: editableTextKey,
                    rendererIgnoresPointer: true,
                    controller: widget.controller,
                    focusNode: widget.focusNode,
                    style: typography.body,
                    cursorColor: colors.accent,
                    backgroundCursorColor: colors.textMuted,
                    selectionColor: colors.accent.withValues(alpha: 0.3),
                    cursorRadius: const Radius.circular(1),
                    cursorHeight: asked,
                    keyboardType: widget.keyboardType,
                    textInputAction: widget.textInputAction,
                    textCapitalization: widget.textCapitalization,
                    autocorrect: widget.autocorrect,
                    enableSuggestions: widget.autocorrect,
                    keyboardAppearance: Theme.brightnessOf(context),
                    selectionControls: selectionControls,
                    contextMenuBuilder: (context, state) =>
                        KansoTextContextMenu(state: state),
                    onChanged: widget.onChanged,
                    onSubmitted: widget.onSubmitted,
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
