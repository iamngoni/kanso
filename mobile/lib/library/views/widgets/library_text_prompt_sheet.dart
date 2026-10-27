// Kanso — library_text_prompt_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_input.dart';

/// A sheet with one line to type, as the desktop's rename and new-tag
/// prompts are: a [title], the text it starts with, and a button that gives
/// the text back.
class LibraryTextPromptSheet extends StatefulWidget {
  const LibraryTextPromptSheet({
    required this.title,
    required this.confirmLabel,
    this.initial = '',
    this.hint,
    super.key,
  });

  final String title;
  final String confirmLabel;
  final String initial;
  final String? hint;

  /// Shows the prompt over the whole app; the text typed, or null when it
  /// was cancelled.
  static Future<String?> show(
    BuildContext context, {
    required String title,
    required String confirmLabel,
    String initial = '',
    String? hint,
  }) {
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => LibraryTextPromptSheet(
        title: title,
        confirmLabel: confirmLabel,
        initial: initial,
        hint: hint,
      ),
    );
  }

  @override
  State<LibraryTextPromptSheet> createState() => _LibraryTextPromptSheetState();
}

class _LibraryTextPromptSheetState extends State<LibraryTextPromptSheet> {
  late final TextEditingController text = TextEditingController(
    text: widget.initial,
  );
  final FocusNode focus = FocusNode(debugLabel: 'Library name');

  @override
  void initState() {
    super.initState();
    // The text starts chosen, as a rename does on the desktop.
    text.selection = TextSelection(
      baseOffset: 0,
      extentOffset: widget.initial.length,
    );
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        focus.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    text.dispose();
    focus.dispose();
    super.dispose();
  }

  void submit() {
    final value = text.text.trim();
    if (value.isNotEmpty) {
      Navigator.of(context).pop(value);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    // The keyboard comes up under the sheet, which already has the home
    // indicator's room.
    final keyboard = math.max<double>(
      0,
      MediaQuery.viewInsetsOf(context).bottom -
          MediaQuery.paddingOf(context).bottom,
    );
    return Padding(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md + keyboard,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  widget.title,
                  style: context.typography.barTitle,
                ),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).maybePop(),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.searchCancel,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoTextInput(
            controller: text,
            focusNode: focus,
            hint: widget.hint,
            semanticLabel: widget.hint,
            textCapitalization: TextCapitalization.words,
            onSubmitted: (_) => submit(),
          ),
          const SizedBox(height: KansoSpacing.sm),
          ListenableBuilder(
            listenable: text,
            builder: (context, _) {
              return KansoButton(
                label: widget.confirmLabel,
                onPressed: text.text.trim().isEmpty ? null : submit,
              );
            },
          ),
        ],
      ),
    );
  }
}
