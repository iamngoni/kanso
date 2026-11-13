// Kanso — when_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../../core/bridge/api/types.dart';

/// The typed-date parser of the Rust core, which the desktop's editor, Quick
/// Capture and the API share: the hint for a phrase at the end of a task
/// and the `@` date list.
///
/// Both answer at once, on the device's clock, and answer null for anything
/// they do not understand. A hint is never worth an error on screen.
abstract interface class WhenRepository {
  /// The hint for [line], one line of the note: where the phrase is, how it
  /// reads and what replaces it. Null when the line is not an open task
  /// ending in a date. [monthFirst] reads `11/14` as November 14th.
  WhenHint? hint(String line, {required bool monthFirst});

  /// The `@` list for [beforeCaret], the text of the caret's line up to the
  /// caret; null when the caret is not completing a date.
  DateChoices? choices(String beforeCaret);
}
