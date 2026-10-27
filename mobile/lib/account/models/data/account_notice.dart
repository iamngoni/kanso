// Kanso — account_notice.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'account_notice.freezed.dart';

/// What the Account page says once a change went through.
@freezed
sealed class AccountNotice with _$AccountNotice {
  /// Another device was signed out.
  const factory AccountNotice.deviceSignedOut({required String name}) =
      AccountNoticeDeviceSignedOut;

  /// A passkey was added; it can unlock a new device, or only sign in.
  const factory AccountNotice.passkeyAdded({required bool canUnlock}) =
      AccountNoticePasskeyAdded;

  const factory AccountNotice.passkeyRemoved() = AccountNoticePasskeyRemoved;

  const factory AccountNotice.providerLinked({required ProviderKind provider}) =
      AccountNoticeProviderLinked;

  const factory AccountNotice.providerUnlinked({
    required ProviderKind provider,
  }) = AccountNoticeProviderUnlinked;
}
