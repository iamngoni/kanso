// Kanso — kanso_spell_check_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_text_menu_toolbar.dart';
import '../controllers/kanso_markdown_controller.dart';

/// The menu over a misspelled word: up to three of the spell checker's
/// suggestions and Delete, in Kanso's menu. Nothing over a word in code, an
/// address or the like, which is not marked.
///
/// It is made for the spell check configuration's
/// `spellCheckSuggestionsToolbarBuilder`.
class KansoSpellCheckMenu extends StatelessWidget {
  const KansoSpellCheckMenu({required this.state, super.key});

  final EditableTextState state;

  /// The most suggestions it offers.
  static const int maxSuggestions = 3;

  @override
  Widget build(BuildContext context) {
    final span = state.findSuggestionSpanAtCursorIndex(
      state.currentTextEditingValue.selection.baseOffset,
    );
    final controller = state.widget.controller;
    if (span == null ||
        (controller is KansoMarkdownController &&
            controller.isQuietAt(span.range.start))) {
      return const SizedBox.shrink();
    }
    void replaceWith(String text) {
      if (!state.mounted) {
        return;
      }
      final value = state.textEditingValue.replaced(span.range, text);
      state.userUpdateTextEditingValue(value, SelectionChangedCause.toolbar);
      SchedulerBinding.instance.addPostFrameCallback((_) {
        if (state.mounted) {
          state.bringIntoView(state.textEditingValue.selection.extent);
        }
      });
      state.hideToolbar();
    }

    return KansoTextMenuToolbar(
      anchors: KansoTextMenuToolbar.anchorsOf(state),
      buttons: <(String, VoidCallback)>[
        for (final suggestion in span.suggestions.take(maxSuggestions))
          (suggestion, () => replaceWith(suggestion)),
        (context.l10n.menuDelete, () => replaceWith('')),
      ],
    );
  }
}
