// Kanso — provider_code.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'provider_code.freezed.dart';

/// The code GitHub asks for, and the page to type it in.
@freezed
abstract class ProviderCode with _$ProviderCode {
  const factory ProviderCode({
    required String userCode,
    required String verificationUri,
  }) = _ProviderCode;
}
