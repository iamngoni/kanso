// Kanso — triage_state.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'triage_bloc.dart';

@freezed
abstract class TriageState with _$TriageState {
  const factory TriageState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,

    /// Every card, skipped ones included.
    @Default(<TriageCardState>[]) List<TriageCardState> cards,

    /// How many notes wait and what is switched on, as last read.
    TriageStatus? info,

    /// What "File all accepted" would do, until the person answers.
    TriageFiled? plan,

    /// Why the last edit or filing did not happen, until it is dismissed.
    String? notice,
  }) = _TriageState;

  const TriageState._();

  /// The cards still showing: not skipped.
  List<TriageCardState> get shown => <TriageCardState>[
    for (final c in cards)
      if (!c.skipped) c,
  ];

  /// The cards with something accepted, which "File all accepted" files.
  List<TriageCardState> get fileable => <TriageCardState>[
    for (final c in shown)
      if (c.hasAccepted && !c.changed) c,
  ];
}
