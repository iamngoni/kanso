// Kanso — dictated_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'dictated_text.freezed.dart';

/// What a transcript becomes in the note: the text that goes in, and where
/// the caret goes in it.
@freezed
abstract class DictatedText with _$DictatedText {
  const factory DictatedText({
    required String text,

    /// The caret's place in [text], in UTF-16 units. The editor puts the
    /// caret after the dictated text; a caret inside it is for the editors
    /// that can place one.
    required int caretUtf16,
  }) = _DictatedText;
}
