// Kanso: opened_batch.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'opened_batch.freezed.dart';

/// The documents the system handed to the app, waiting to be opened.
@freezed
abstract class OpenedBatch with _$OpenedBatch {
  const factory OpenedBatch({
    /// The documents, in the order they arrived.
    required List<OpenedFile> files,

    /// The folder the library stages copies of them under.
    required String staging,
  }) = _OpenedBatch;
}
