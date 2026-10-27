// Kanso — account_action_field.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'account_action_field.freezed.dart';

/// A field of the sheet that asks for what a change to the account needs:
/// the current password, a new one, the account's name.
@freezed
abstract class AccountActionField with _$AccountActionField {
  const factory AccountActionField({
    required String label,

    /// Whether what is typed is hidden.
    @Default(true) bool obscure,
  }) = _AccountActionField;
}
