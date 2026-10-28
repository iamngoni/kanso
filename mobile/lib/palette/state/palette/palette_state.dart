// Kanso — palette_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'palette_bloc.dart';

@freezed
abstract class PaletteState with _$PaletteState {
  const factory PaletteState({
    /// What is typed.
    @Default('') String query,

    /// Where the results are.
    @Default(PaletteStatus.idle) PaletteStatus status,

    /// What matches, in the order the palette shows it.
    @Default(<PaletteSection>[]) List<PaletteSection> sections,

    /// Why the last search failed, to say instead of the list.
    String? problem,
  }) = _PaletteState;

  const PaletteState._();

  /// Nothing matches what is typed.
  bool get isEmpty =>
      status == PaletteStatus.ready &&
      sections.every((section) => section.entries.isEmpty);
}
