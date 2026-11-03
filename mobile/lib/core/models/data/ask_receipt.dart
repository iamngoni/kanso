// Kanso — ask_receipt.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ask_receipt.freezed.dart';

/// The lines of a note one sentence of an answer rests on, as they were sent
/// to the model.
@freezed
abstract class AskReceipt with _$AskReceipt {
  const factory AskReceipt({
    /// Its number in the answer, from 1.
    required int n,
    required String noteId,
    @Default('') String title,

    /// "Pricing · lines 12–14 · 14 Oct".
    @Default('') String label,

    /// The first and last line cited, from 0.
    @Default(0) int start,
    @Default(0) int end,

    /// The cited lines as they were sent.
    @Default('') String quote,
  }) = _AskReceipt;
}
