// Kanso — kanso_text_field.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_search_input.dart' show KansoSearchInput;
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A field for a line of text with its [label] above it: a username, a
/// password, a server's address, a recovery key.
///
/// It is drawn as the search field is, with the accent outline while it has
/// focus (the danger colour while [invalid]), over Flutter's `EditableText`
/// with Kanso's own caret and context menu, so iOS and Android draw it
/// alike. The text is the [controller]'s, which the caller owns.
///
/// A [obscure] field shows dots. [mono] sets the text in the code font, for a
/// key. Without [enabled] the field is dimmed and takes no input. Under it
/// is the [error], in the danger colour, or else the [help].
class KansoTextField extends StatefulWidget {
  const KansoTextField({
    required this.controller,
    required this.label,
    this.hint,
    this.obscure = false,
    this.mono = false,
    this.invalid = false,
    this.help,
    this.error,
    this.enabled = true,
    this.autofocus = false,
    this.keyboardType = TextInputType.text,
    this.textInputAction,
    this.autofillHints,
    this.inputFormatters,
    this.onChanged,
    this.onSubmitted,
    super.key,
  });

  final TextEditingController controller;
  final String label;
  final String? hint;
  final bool obscure;
  final bool mono;
  final bool invalid;

  /// Advice under the field, such as how long a password must be.
  final String? help;

  /// What is wrong with the text, under the field in the danger colour; the
  /// field is drawn as [invalid] while there is one.
  final String? error;
  final bool enabled;
  final bool autofocus;
  final TextInputType keyboardType;
  final TextInputAction? textInputAction;
  final Iterable<String>? autofillHints;
  final List<TextInputFormatter>? inputFormatters;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;

  @override
  State<KansoTextField> createState() => _KansoTextFieldState();
}

class _KansoTextFieldState extends State<KansoTextField>
    implements TextSelectionGestureDetectorBuilderDelegate {
  late final FocusNode focusNode;
  late final TextSelectionGestureDetectorBuilder gestures;

  /// No handles and no menu of the platform's: see [KansoSelectionControls].
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
    focusNode = FocusNode();
    gestures = TextSelectionGestureDetectorBuilder(delegate: this);
  }

  @override
  void dispose() {
    focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    // The same caret on both platforms: see `KansoSearchInput`.
    final caretAsked = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => KansoSearchInput.caretHeight - 2,
      _ => KansoSearchInput.caretHeight,
    };
    final style = widget.mono ? typography.mono : typography.body;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.only(bottom: KansoSpacing.xxs),
          child: Text(widget.label, style: typography.label),
        ),
        ListenableBuilder(
          listenable: Listenable.merge(<Listenable>[
            widget.controller,
            focusNode,
          ]),
          builder: (context, _) {
            final focused = focusNode.hasFocus;
            final empty = widget.controller.text.isEmpty;
            final invalid = widget.invalid || widget.error != null;
            final outline = invalid
                ? colors.danger
                : focused
                ? colors.accent
                : colors.divider;
            return Opacity(
              opacity: widget.enabled ? 1 : KansoEffects.disabled,
              child: Container(
                height: KansoSizes.field,
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.sm,
                ),
                decoration: BoxDecoration(
                  color: colors.elevated,
                  borderRadius: KansoRadii.mediumBorder,
                  border: Border.all(
                    color: outline,
                    width: focused || invalid ? 1.5 : 1,
                  ),
                ),
                child: Stack(
                  alignment: AlignmentDirectional.centerStart,
                  children: <Widget>[
                    if (empty && widget.hint != null)
                      IgnorePointer(
                        child: Text(
                          widget.hint!,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: style.copyWith(color: colors.textMuted),
                        ),
                      ),
                    gestures.buildGestureDetector(
                      behavior: HitTestBehavior.translucent,
                      child: EditableText(
                        key: editableTextKey,
                        rendererIgnoresPointer: true,
                        controller: widget.controller,
                        focusNode: focusNode,
                        autofocus: widget.autofocus,
                        readOnly: !widget.enabled,
                        obscureText: widget.obscure,
                        style: style,
                        cursorColor: colors.accent,
                        backgroundCursorColor: colors.textMuted,
                        selectionColor: colors.accent.withValues(alpha: 0.3),
                        cursorRadius: const Radius.circular(1),
                        cursorHeight: caretAsked,
                        keyboardType: widget.keyboardType,
                        textInputAction: widget.textInputAction,
                        autofillHints: widget.autofillHints,
                        inputFormatters: widget.inputFormatters,
                        autocorrect: false,
                        enableSuggestions: false,
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
        ),
        if (widget.error ?? widget.help case final line?)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.xxs),
            child: Text(
              line,
              style: typography.meta.copyWith(
                color: widget.error == null ? null : colors.danger,
              ),
            ),
          ),
      ],
    );
  }
}
