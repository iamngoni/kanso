// Kanso — publish_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'publish_bloc.dart';

@freezed
sealed class PublishState with _$PublishState {
  const factory PublishState.idle({PublishViewModel? view}) = PublishIdleState;

  const factory PublishState.publishing({required PublishViewModel view}) =
      PublishPublishingState;

  const factory PublishState.published({
    required PublishViewModel view,
    required PublishLink link,
  }) = PublishPublishedState;

  const factory PublishState.failed({
    required PublishViewModel view,
    required String text,
  }) = PublishFailedState;
}
