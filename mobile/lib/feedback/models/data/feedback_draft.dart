// Kanso — feedback_draft.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:freezed_annotation/freezed_annotation.dart';

import 'feedback_kind.dart';

part 'feedback_draft.freezed.dart';

/// The shortest report text, in characters.
const int feedbackMinText = 10;

/// The longest report text, in characters.
const int feedbackMaxText = 4000;

/// The counter shows from here.
const int feedbackCounterFrom = 3500;

/// The largest screenshot, in bytes (5 MB).
const int feedbackMaxScreenshot = 5 * 1024 * 1024;

/// The longest credit name, in characters.
const int feedbackMaxName = 39;

/// An image the person chose to attach.
@freezed
abstract class FeedbackScreenshot with _$FeedbackScreenshot {
  const factory FeedbackScreenshot({
    required Uint8List bytes,

    /// The file's name, for the row that shows it.
    required String name,
  }) = _FeedbackScreenshot;
}

/// A report as the person filled it in. **No note is ever part of it.**
@freezed
abstract class FeedbackDraft with _$FeedbackDraft {
  const factory FeedbackDraft({
    @Default(FeedbackKind.bug) FeedbackKind kind,
    @Default('') String text,

    /// Whether "Credit me by name" is on.
    @Default(false) bool credit,
    @Default('') String creditName,
    @Default(true) bool includeApp,
    FeedbackScreenshot? screenshot,
  }) = _FeedbackDraft;

  const FeedbackDraft._();

  /// The characters typed.
  int get length => text.runes.length;

  /// Whether Send is on: ten characters or more, not too many, and a valid
  /// name when credit is asked for.
  bool get canSend =>
      length >= feedbackMinText &&
      length <= feedbackMaxText &&
      (!credit || creditNameValid);

  /// Whether the credit name is 1 to 39 letters, numbers, spaces, dots,
  /// dashes or underscores, and not blank.
  bool get creditNameValid {
    final name = creditName.trim();
    final count = name.runes.length;
    return count >= 1 &&
        count <= feedbackMaxName &&
        RegExp(r'^[\p{L}\p{N} ._-]+$', unicode: true).hasMatch(name);
  }

  /// Whether the counter shows.
  bool get showsCounter => length >= feedbackCounterFrom;
}
