// Kanso — register_lock_blocs.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/lock/models/repos/abs/device_lock_repository.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso/lock/state/change_password/change_password_bloc.dart';
import 'package:kanso/lock/state/lock_remove/lock_remove_bloc.dart';
import 'package:kanso/lock/state/lock_setup/lock_setup_bloc.dart';
import 'package:kanso/lock/state/unlock/unlock_bloc.dart';

import 'seeded_appearance_bloc.dart';

void _register<T extends Object>(T Function() make) {
  if (getIt.isRegistered<T>()) {
    getIt.unregister<T>();
  }
  getIt.registerFactory<T>(make);
  addTearDown(() {
    if (getIt.isRegistered<T>()) {
      getIt.unregister<T>();
    }
  });
}

/// Registers, for a test, the BLoCs the lock sheets make for themselves over
/// the given [session] and [device]; they are taken out again when the test
/// ends.
void registerLockBlocs(
  SessionRepository session,
  DeviceLockRepository device, [
  PreferencesRepository? preferences,
]) {
  final saved = preferences ?? ForgetfulPreferencesRepository();
  _register<UnlockBloc>(() => UnlockBloc(session, device, saved));
  _register<LockSetupBloc>(() => LockSetupBloc(session));
  _register<LockRemoveBloc>(() => LockRemoveBloc(session));
  _register<ChangePasswordBloc>(() => ChangePasswordBloc(session));
  _register<BiometricsBloc>(() => BiometricsBloc(session, device));
}
