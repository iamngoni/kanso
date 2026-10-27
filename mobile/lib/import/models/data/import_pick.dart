// Kanso: import_pick.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'import_pick.freezed.dart';

/// A file or folder the person chose, as a copy the app can read.
///
/// The system's pickers hand over places the app has no right to read
/// directly, so the native side copies the choice into the app's cache and
/// this is where the copy is.
@freezed
abstract class ImportPick with _$ImportPick {
  const factory ImportPick({
    /// Where the copy is: a file, or a folder with its contents.
    required String path,

    /// What the person chose, by its own name.
    required String name,
  }) = _ImportPick;
}
