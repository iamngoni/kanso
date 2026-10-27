// Kanso — theme_file_notice.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_file_notice.freezed.dart';

/// Why a theme file in the themes folder was not used.
@freezed
abstract class ThemeFileNotice with _$ThemeFileNotice {
  const factory ThemeFileNotice({
    /// The file's name.
    required String file,

    /// What is wrong with it, in a sentence.
    required String sentence,
  }) = _ThemeFileNotice;
}
