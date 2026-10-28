// Kanso — palette_choice.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'palette_choice.freezed.dart';

/// One line of what a palette command asks for when it needs a value: a
/// status, a notebook, a tag.
@freezed
abstract class PaletteChoice with _$PaletteChoice {
  const factory PaletteChoice({
    /// What the line says.
    required String label,

    /// What choosing it gives the command.
    required String value,

    /// Whether the note already has it.
    @Default(false) bool checked,

    /// How far a notebook nests, for the lines that nest.
    @Default(0) int depth,
  }) = _PaletteChoice;
}
