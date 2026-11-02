// Kanso — triage_card_state.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/triage_types.dart';

part 'triage_card_state.freezed.dart';

/// Where a chip stands: outlined (untouched), filled (accepted) or struck
/// through (rejected, and remembered so it does not come back).
enum ChipMark { untouched, accepted, rejected }

/// One chip on a card, with what the person did to it.
@freezed
abstract class TriageChipState with _$TriageChipState {
  const factory TriageChipState({
    required TriageChip chip,
    @Default(ChipMark.untouched) ChipMark mark,

    /// Whether the person changed the value.
    @Default(false) bool edited,
  }) = _TriageChipState;
}

/// One Inbox note with its chips. The rules for the marks are the desktop's
/// (`kanso_engine::triage::session`): a notebook and a title are accepted
/// one at a time; a rejected chip is never filed.
@freezed
abstract class TriageCardState with _$TriageCardState {
  const factory TriageCardState({
    required String noteId,

    /// The revision the suggestions were made from.
    required String rev,
    required String title,
    required String snippet,
    required int updatedMs,
    required List<TriageChipState> chips,
    @Default(TriageMissing.nothing) TriageMissing missing,

    /// The note changed on another device: the card is read again and File
    /// waits until it has been.
    @Default(false) bool changed,

    /// Hidden for this visit.
    @Default(false) bool skipped,

    /// Why the AI's suggestions are missing, when they are.
    String? aiError,
  }) = _TriageCardState;

  const TriageCardState._();

  /// A card as the library made it, every chip untouched.
  factory TriageCardState.fromCard(TriageCard card) => TriageCardState(
    noteId: card.noteId,
    rev: card.rev,
    title: card.title,
    snippet: card.snippet,
    updatedMs: card.updatedMs,
    chips: <TriageChipState>[
      for (final chip in card.chips) TriageChipState(chip: chip),
    ],
    missing: card.missing,
  );

  /// The chips the person accepted, in the card's order.
  List<TriageChip> get accepted => <TriageChip>[
    for (final c in chips)
      if (c.mark == ChipMark.accepted) c.chip,
  ];

  bool get hasAccepted => chips.any((c) => c.mark == ChipMark.accepted);

  /// What File applies; null while the card says the note changed.
  TriageFiling? get filing => changed
      ? null
      : TriageFiling(noteId: noteId, rev: rev, accepted: accepted);

  /// Accepts chip [index], or lifts the acceptance. A notebook or a title
  /// accepted un-accepts the other one.
  TriageCardState toggleAccept(int index) {
    if (index < 0 || index >= chips.length) {
      return this;
    }
    final field = chips[index].chip.field;
    if (chips[index].mark == ChipMark.accepted) {
      return _withMark(index, ChipMark.untouched);
    }
    final exclusive = _exclusive(field);
    return copyWith(
      chips: <TriageChipState>[
        for (var i = 0; i < chips.length; i++)
          if (i == index)
            chips[i].copyWith(mark: ChipMark.accepted)
          else if (exclusive &&
              chips[i].chip.field == field &&
              chips[i].mark == ChipMark.accepted)
            chips[i].copyWith(mark: ChipMark.untouched)
          else
            chips[i],
      ],
    );
  }

  /// Rejects chip [index], or lifts the rejection.
  TriageCardState toggleReject(int index) {
    if (index < 0 || index >= chips.length) {
      return this;
    }
    return _withMark(
      index,
      chips[index].mark == ChipMark.rejected
          ? ChipMark.untouched
          : ChipMark.rejected,
    );
  }

  /// Accepts every chip that isn't rejected: one notebook and one title at
  /// most, the first of each.
  TriageCardState acceptAll() => _acceptWhere((_) => true);

  /// Accepts every chip of [field] that isn't rejected (the first, for a
  /// notebook or a title).
  TriageCardState acceptField(TriageField field) =>
      _acceptWhere((c) => c.field == field);

  /// Replaces chip [index] with [chip], as the person chose or typed it,
  /// and accepts it. Another chip with the same value is dropped.
  TriageCardState edit(int index, TriageChip chip) {
    if (index < 0 || index >= chips.length) {
      return this;
    }
    return copyWith(
      chips: <TriageChipState>[
        for (var i = 0; i < chips.length; i++)
          if (i == index)
            TriageChipState(chip: chip, mark: ChipMark.accepted, edited: true)
          else if (chips[i].chip.field != chip.field ||
              chips[i].chip.value != chip.value)
            chips[i],
      ],
    );
  }

  /// Replaces the chips with [fresh] (the model's ranking, or the card made
  /// again), keeping the marks of the chips still there.
  TriageCardState withChips(List<TriageChip> fresh) => copyWith(
    chips: <TriageChipState>[
      for (final chip in fresh)
        TriageChipState(
          chip: chip,
          mark:
              chips
                  .where(
                    (c) =>
                        c.chip.field == chip.field &&
                        c.chip.value == chip.value,
                  )
                  .map((c) => c.mark)
                  .firstOrNull ??
              ChipMark.untouched,
        ),
    ],
  );

  TriageCardState _withMark(int index, ChipMark mark) => copyWith(
    chips: <TriageChipState>[
      for (var i = 0; i < chips.length; i++)
        if (i == index) chips[i].copyWith(mark: mark) else chips[i],
    ],
  );

  TriageCardState _acceptWhere(bool Function(TriageChip) test) {
    final seen = <TriageField>{};
    return copyWith(
      chips: <TriageChipState>[
        for (final c in chips)
          if (c.mark == ChipMark.rejected || !test(c.chip))
            c
          else if (_exclusive(c.chip.field))
            c.copyWith(
              mark: seen.add(c.chip.field)
                  ? ChipMark.accepted
                  : ChipMark.untouched,
            )
          else
            c.copyWith(mark: ChipMark.accepted),
      ],
    );
  }

  static bool _exclusive(TriageField field) =>
      field == TriageField.notebook || field == TriageField.title;
}
