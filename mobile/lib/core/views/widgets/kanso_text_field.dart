// Kanso — kanso_text_field.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../configs/kanso_colors.dart';
import '../../configs/kanso_effects.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../configs/kanso_typography.dart';
import '../../utils/context_extensions.dart';
import 'kanso_search_input.dart' show KansoSearchInput;
import 'kanso_selection_controls.dart';
import 'kanso_text_context_menu.dart';

/// A field for text in the app's type: a caret and selection drawn the same
/// on every platform, a hint while empty, and nothing around it. A row or a
/// box around it gives it its shape.
///
/// The caller owns [controller]. It is one line unless [maxLines] says
/// more, and obscured while [obscure] is set (a key), where nothing is
/// suggested or corrected.
class KansoTextField extends StatefulWidget {
  const KansoTextField({
    required this.controller,
    this.label,
    this.hint,
    this.onChanged,
    this.onSubmitted,
    this.obscure = false,
    this.mono = false,
    this.invalid = false,
    this.help,
    this.error,
    this.enabled = true,
    this.autofillHints,
    this.inputFormatters,
    this.minLines = 1,
    this.maxLines = 1,
    this.keyboardType = TextInputType.text,
    this.textInputAction,
    this.autocorrect = false,
    this.autofocus = false,
    this.semanticLabel,
    super.key,
  }) : assert(label != null || hint != null, 'Give the field a label or hint.');

  final TextEditingController controller;

  /// The labelled, outlined form retained for Settings, account and lock
  /// forms that already use this control.
  final String? label;
  final String? hint;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final bool obscure;
  final bool mono;
  final bool invalid;
  final String? help;
  final String? error;
  final bool enabled;
  final Iterable<String>? autofillHints;
  final List<TextInputFormatter>? inputFormatters;
  final int minLines;
  final int maxLines;
  final TextInputType keyboardType;
  final TextInputAction? textInputAction;
  final bool autocorrect;
  final bool autofocus;

  /// What a screen reader says for the field.
  final String? semanticLabel;

  /// How tall the caret is, in points.
  static const double caretHeight = 18;

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
    if (widget.label != null) {
      return legacyField(context, colors, typography);
    }
    // The framework sizes the caret differently on Apple's platforms and on
    // Android's; the same is asked for on both so the pictures match.
    final caret = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => KansoTextField.caretHeight - 2,
      _ => KansoTextField.caretHeight,
    };
    return Semantics(
      label: widget.semanticLabel ?? widget.label,
      textField: true,
      child: ListenableBuilder(
        listenable: widget.controller,
        builder: (context, _) {
          return Stack(
            children: <Widget>[
              if (widget.controller.text.isEmpty && widget.hint != null)
                IgnorePointer(
                  child: Text(
                    widget.hint!,
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
                  focusNode: focusNode,
                  autofocus: widget.autofocus,
                  readOnly: !widget.enabled,
                  style: typography.body,
                  cursorColor: colors.accent,
                  backgroundCursorColor: colors.textMuted,
                  selectionColor: colors.accent.withValues(alpha: 0.3),
                  cursorRadius: const Radius.circular(1),
                  cursorHeight: caret,
                  minLines: widget.minLines,
                  maxLines: widget.maxLines,
                  obscureText: widget.obscure,
                  autofillHints: widget.autofillHints,
                  inputFormatters: widget.inputFormatters,
                  keyboardType: widget.maxLines == 1
                      ? widget.keyboardType
                      : TextInputType.multiline,
                  textInputAction: widget.textInputAction,
                  autocorrect:
                      widget.autocorrect && !widget.obscure && widget.enabled,
                  enableSuggestions:
                      widget.autocorrect && !widget.obscure && widget.enabled,
                  selectionControls: selectionControls,
                  contextMenuBuilder: (context, state) =>
                      KansoTextContextMenu(state: state),
                  onChanged: widget.onChanged,
                  onSubmitted: widget.onSubmitted,
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  /// The existing labelled control's appearance and validation behavior.
  Widget legacyField(
    BuildContext context,
    KansoColors colors,
    KansoTypography typography,
  ) {
    final fieldStyle = widget.mono ? typography.mono : typography.body;
    final caret = switch (Theme.of(context).platform) {
      TargetPlatform.iOS ||
      TargetPlatform.macOS => KansoSearchInput.caretHeight - 2,
      _ => KansoSearchInput.caretHeight,
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.only(bottom: KansoSpacing.xxs),
          child: Text(widget.label!, style: typography.label),
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
                          style: fieldStyle.copyWith(color: colors.textMuted),
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
                        style: fieldStyle,
                        cursorColor: colors.accent,
                        backgroundCursorColor: colors.textMuted,
                        selectionColor: colors.accent.withValues(alpha: 0.3),
                        cursorRadius: const Radius.circular(1),
                        cursorHeight: caret,
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
