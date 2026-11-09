// Kanso — settings_navigation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:go_router/go_router.dart';

import '../bridge/api/types.dart';
import 'kanso_routes.dart';

/// Moving about Settings once a flow of the account is over.
extension SettingsNavigation on GoRouter {
  /// The address of the page on top of the stack.
  ///
  /// Read from the match list's own `last`, which goes down into a shell
  /// route and into a page that was pushed. The last of `matches` is the
  /// shell itself (Settings is one) and keeps the address Settings was
  /// opened at, so after `push` it still said Settings while a page of the
  /// account was on top of it.
  String get topLocation =>
      routerDelegate.currentConfiguration.last.matchedLocation;

  /// Closes the pages on top of Settings itself, so the flow that was run
  /// from some of them leaves Settings as it was.
  void popToSettings() {
    while (canPop() && topLocation != KansoRoutes.settings) {
      pop();
    }
  }

  /// Closes the flow and shows [key], the recovery key it made, on Settings.
  void popToRecoveryKey(RecoveryKeyText key) {
    popToSettings();
    unawaited(push<void>(KansoRoutes.settingsRecoveryKey, extra: key));
  }
}
