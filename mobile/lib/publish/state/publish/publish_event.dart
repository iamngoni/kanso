// Kanso — publish_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'publish_bloc.dart';

@freezed
sealed class PublishEvent with _$PublishEvent {
  const factory PublishEvent.openNote({
    required String id,
    required PaletteDto palette,
    required PreviewOptions options,
  }) = PublishOpenNoteEvent;

  const factory PublishEvent.openNotebook({
    required String id,
    required PaletteDto palette,
    required PreviewOptions options,
  }) = PublishOpenNotebookEvent;

  const factory PublishEvent.publish(PublishExpiry expiry) = PublishSubmitEvent;

  const factory PublishEvent.unpublish() = PublishUnpublishEvent;

  const factory PublishEvent.expiryPicked(PublishExpiry expiry) =
      PublishExpiryPickedEvent;
}
