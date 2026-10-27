// Kanso — text_diff.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'text_diff.freezed.dart';

/// What changed between two texts: [text] now stands where [start] to [end]
/// of the old text was. Offsets are UTF-16 units of the old text.
@freezed
abstract class TextDiff with _$TextDiff {
  const factory TextDiff({
    required int start,
    required int end,
    required String text,
  }) = _TextDiff;

  const TextDiff._();

  /// Whether nothing changed.
  bool get isEmpty => start == end && text.isEmpty;

  /// Whether text was only put in, none taken out.
  bool get isInsertion => start == end && text.isNotEmpty;
}
