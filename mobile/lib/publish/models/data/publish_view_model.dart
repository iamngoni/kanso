// Kanso — publish_view_model.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/publish.dart';

part 'publish_view_model.freezed.dart';

/// The content summary shared by the note and notebook Publish sheets.
@freezed
abstract class PublishViewModel with _$PublishViewModel {
  const factory PublishViewModel({
    required String id,
    required String title,
    required bool notebook,
    required String summary,
    @Default('') String contentPreview,
    @Default(false) bool needsUpdate,
    @Default(PublishExpiry.never) PublishExpiry expiry,
    @Default(<PublishExpiryOption>[]) List<PublishExpiryOption> expiryOptions,
    @Default('') String locked,
    String? refusal,
    PublishLink? link,
  }) = _PublishViewModel;
}
