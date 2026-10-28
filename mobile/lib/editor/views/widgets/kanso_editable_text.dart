// Kanso — kanso_editable_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart' show DragStartBehavior;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../utils/shift_runs.dart';
import '../controllers/kanso_markdown_controller.dart';

/// Flutter's `EditableText` for the note's Markdown, with its text drawn by
/// the [KansoMarkdownController].
///
/// `EditableText` draws the controller's own spans only until the platform's
/// spell checker has answered; from then on it draws its own, without
/// asking the controller, and the colours of the text would go. This draws
/// the controller's spans always, with the misspellings merged in.
///
/// It is a thin subclass; it depends on `EditableTextState.buildTextSpan` and
/// `spellCheckResults`, and the widget tests of the editor cover each path.
class KansoEditableText extends EditableText {
  KansoEditableText({
    required KansoMarkdownController super.controller,
    required super.focusNode,
    required super.style,
    required super.cursorColor,
    required super.backgroundCursorColor,
    required super.scrollPadding,
    required super.keyboardAppearance,
    required super.selectionColor,
    required super.selectionControls,
    required super.contextMenuBuilder,
    required super.spellCheckConfiguration,
    required super.magnifierConfiguration,
    required super.cursorHeight,
    required super.autofocus,
    super.key,
    super.cursorRadius,
    super.cursorOffset,
    super.scrollController,
    super.onChanged,
    super.undoController,
    super.readOnly,
  }) : super(
         maxLines: null,
         minLines: 1,
         // The gestures around it place the caret and select.
         rendererIgnoresPointer: true,
         keyboardType: TextInputType.multiline,
         textInputAction: TextInputAction.newline,
         textCapitalization: TextCapitalization.sentences,
         // Smart punctuation would turn `"` and `--` into marks that
         // Markdown does not know.
         smartQuotesType: SmartQuotesType.disabled,
         smartDashesType: SmartDashesType.disabled,
         autocorrect: true,
         enableSuggestions: true,
         stylusHandwritingEnabled: true,
         dragStartBehavior: DragStartBehavior.down,
       );

  @override
  KansoEditableTextState createState() => KansoEditableTextState();
}

/// The state of [KansoEditableText].
class KansoEditableTextState extends EditableTextState {
  bool _placeholder = false;
  SpellCheckResults? _shiftedFrom;
  String? _shiftedTo;
  List<TextRange> _shifted = const <TextRange>[];

  @override
  void insertTextPlaceholder(Size size) {
    _placeholder = true;
    super.insertTextPlaceholder(size);
  }

  @override
  void removeTextPlaceholder() {
    _placeholder = false;
    super.removeTextPlaceholder();
  }

  /// Select All of a long note makes the window the whole note first: one
  /// slow layout, for something the person asked for.
  @override
  void selectAll(SelectionChangedCause cause) {
    final controller = widget.controller;
    if (controller is KansoMarkdownController && controller.isWindowed) {
      controller.selectEverything();
    }
    super.selectAll(cause);
  }

  @override
  TextSpan buildTextSpan() {
    final controller = widget.controller;
    // Dictation and handwriting put a placeholder in the text: Flutter draws
    // that.
    if (controller is! KansoMarkdownController || _placeholder) {
      return super.buildTextSpan();
    }
    return controller.spanFor(
      style: widget.style,
      withComposing: !widget.readOnly && widget.focusNode.hasFocus,
      misspellings: _misspellings(controller.text),
    );
  }

  /// The misspelled ranges of the last spell check, moved to fit [text] when
  /// it has changed since: a word being typed is not marked until it is
  /// checked again.
  List<TextRange> _misspellings(String text) {
    final results = spellCheckResults;
    if (results == null || results.suggestionSpans.isEmpty) {
      return const <TextRange>[];
    }
    if (identical(results, _shiftedFrom) && identical(text, _shiftedTo)) {
      return _shifted;
    }
    var ranges = <TextRange>[
      for (final span in results.suggestionSpans) span.range,
    ];
    final old = results.spellCheckedText;
    if (old != text) {
      var start = 0;
      final shortest = old.length < text.length ? old.length : text.length;
      while (start < shortest &&
          old.codeUnitAt(start) == text.codeUnitAt(start)) {
        start++;
      }
      var tail = 0;
      while (tail < shortest - start &&
          old.codeUnitAt(old.length - 1 - tail) ==
              text.codeUnitAt(text.length - 1 - tail)) {
        tail++;
      }
      ranges = shiftRanges(
        ranges,
        start: start,
        end: old.length - tail,
        inserted: text.length - tail - start,
      );
    }
    _shiftedFrom = results;
    _shiftedTo = text;
    _shifted = ranges;
    return ranges;
  }
}
