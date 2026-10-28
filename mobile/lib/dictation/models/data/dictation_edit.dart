// Kanso — dictation_edit.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'dictation_edit.freezed.dart';

/// One change dictation makes to the note: [text] in place of [range].
@freezed
abstract class DictationEdit with _$DictationEdit {
  const factory DictationEdit({
    /// What the edit replaces, in the note's text now.
    required TextRange range,

    /// What goes in its place, spacing included.
    required String text,
  }) = _DictationEdit;
}
