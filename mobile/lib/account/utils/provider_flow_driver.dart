// Kanso — provider_flow_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/data/application_error.dart';
import '../../core/models/repos/abs/auth_session_repository.dart';

/// Does for the app what a sign-in with Apple, Google or GitHub (or linking
/// one) asks of it: opens the provider's page in the system's auth session,
/// closes it, and tells the library how the session ended.
///
/// One driver is for one flow. The library's events go to [handle]; what the
/// screen shows of them (the code, the username question, the end) is the
/// BLoC's.
class ProviderFlowDriver {
  ProviderFlowDriver(this._session, this._pageEnded);

  final AuthSessionRepository _session;
  final Future<Either<ApplicationError, Unit>> Function(String? callbackUrl)
  _pageEnded;

  /// Whether the app itself ended the page: the session's `open` then
  /// completes with null, which is not the person turning away from it.
  bool _closedByApp = false;

  /// Opens or closes the page as [event] asks.
  void handle(ProviderEvent event) {
    switch (event) {
      case ProviderEvent_OpenPage(:final url, :final callbackScheme):
        _closedByApp = false;
        unawaited(_open(url, callbackScheme));
      case ProviderEvent_ClosePage():
      case ProviderEvent_NeedsUsername():
      case ProviderEvent_Finished():
      case ProviderEvent_Failed():
      case ProviderEvent_Cancelled():
        _closedByApp = true;
        unawaited(_session.close());
      case ProviderEvent_ShowCode():
        break;
    }
  }

  Future<void> _open(String url, String? callbackScheme) async {
    final result = await _session.open(url, callbackScheme: callbackScheme);
    // The app closed it because the flow no longer needs it: the library has
    // what it wanted, and must not be told that the person turned away.
    if (_closedByApp) {
      return;
    }
    // A page that could not be opened ends the flow like a dismissed one.
    final callback = result.fold((_) => null, (url) => url);
    await _pageEnded(callback);
  }
}
