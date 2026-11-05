// Kanso — pdf_opening.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/pdf_types.dart';
import '../repos/abs/pdf_reader_repository.dart';

part 'pdf_opening.freezed.dart';

/// What opening a PDF came to.
@freezed
sealed class PdfOpening with _$PdfOpening {
  /// Open: the document, and what it holds.
  const factory PdfOpening.opened({
    required PdfDocumentSession session,
    required PdfInfo info,
  }) = PdfOpened;

  /// Not opened, and why.
  const factory PdfOpening.declined({required PdfDeclined reason}) =
      PdfDeclinedOpening;
}
