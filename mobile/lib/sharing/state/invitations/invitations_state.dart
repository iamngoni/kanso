// Kanso — invitations_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'invitations_bloc.dart';

@freezed
abstract class InvitationsState with _$InvitationsState {
  const factory InvitationsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The notes shared with this account that wait to be accepted, the
    /// oldest first.
    @Default(<InvitationRow>[]) List<InvitationRow> invitations,

    /// No account is signed in, so none can be shared with it.
    @Default(false) bool signedOut,

    /// The invitations being answered now, by share id.
    @Default(<String>{}) Set<String> working,

    /// Why the last answer did not happen.
    ShareProblem? problem,

    /// The note an invitation just accepted brought in, to open.
    String? opened,
  }) = _InvitationsState;
}
