// Kanso — answer_block.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'answer_block.freezed.dart';

/// A block of an answer's Markdown. Only these are drawn: an answer has no
/// images, no tables and no HTML.
@freezed
sealed class AnswerBlock with _$AnswerBlock {
  /// A heading of [level] 1 to 6; [text] is inline Markdown.
  const factory AnswerBlock.heading({
    required int level,
    required String text,
  }) = AnswerHeading;

  /// A paragraph of inline Markdown.
  const factory AnswerBlock.paragraph(String text) = AnswerParagraph;

  /// A bulleted or numbered list.
  const factory AnswerBlock.list(List<AnswerListItem> items) = AnswerList;

  /// Lines of code, exactly as written.
  const factory AnswerBlock.code(String text) = AnswerCode;

  /// A quotation; [text] is inline Markdown.
  const factory AnswerBlock.quote(String text) = AnswerQuote;

  /// A horizontal rule.
  const factory AnswerBlock.rule() = AnswerRule;
}

/// One item of a list.
@freezed
abstract class AnswerListItem with _$AnswerListItem {
  const factory AnswerListItem({
    /// What leads the item: a bullet, or its number and a full stop.
    required String marker,

    /// The item's inline Markdown.
    required String text,

    /// How deep the item is nested, from 0.
    @Default(0) int depth,
  }) = _AnswerListItem;
}
