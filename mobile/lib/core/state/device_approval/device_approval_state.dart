// Kanso — device_approval_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'device_approval_bloc.dart';

@freezed
abstract class DeviceApprovalState with _$DeviceApprovalState {
  const factory DeviceApprovalState({
    /// The requests waiting, oldest first.
    @Default(<ApprovalPrompt>[]) List<ApprovalPrompt> prompts,

    /// The request being answered, if one is.
    String? answering,

    /// Why the last answer failed.
    ApplicationError? error,
  }) = _DeviceApprovalState;

  const DeviceApprovalState._();

  /// The request the sheet is about: the first that is ready to be answered,
  /// or that changed after it was made (and is to be declined).
  ApprovalPrompt? get shown {
    for (final prompt in prompts) {
      if (prompt.state == PromptState.ready ||
          prompt.state == PromptState.changed) {
        return prompt;
      }
    }
    return null;
  }
}
