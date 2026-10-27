// Kanso — join_link_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'join_link_bloc.dart';

@freezed
abstract class JoinLinkState with _$JoinLinkState {
  const factory JoinLinkState({
    /// The request is being sent.
    @Default(false) bool joining,

    /// Who was asked to let the person in, once the request is sent.
    JoinRow? asked,

    /// Why the request was not sent.
    ShareProblem? problem,
  }) = _JoinLinkState;
}
