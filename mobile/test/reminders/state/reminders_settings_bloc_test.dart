// Kanso — reminders_settings_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/reminder_permission.dart';
import 'package:kanso/reminders/state/reminders_settings/reminders_settings_bloc.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_reminder_notifier.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/seeded_appearance_bloc.dart';

void main() {
  late MockPreferencesRepository preferences;
  late MockRemindersRepository reminders;
  late FixtureReminderNotifier notifier;
  late RemindersCoordinator coordinator;
  var saved = const Preferences();

  setUp(() {
    saved = const Preferences();
    preferences = MockPreferencesRepository();
    reminders = MockRemindersRepository();
    notifier = FixtureReminderNotifier();
    final library = FixtureLibraryRepository();
    addTearDown(library.close);
    addTearDown(notifier.live.close);
    when(() => preferences.load()).thenAnswer(
      (_) async => Right<ApplicationError, Preferences>(saved),
    );
    when(() => preferences.update(any())).thenAnswer((call) async {
      final change =
          call.positionalArguments.first as Preferences Function(Preferences);
      saved = change(saved);
      return Right<ApplicationError, Preferences>(saved);
    });
    when(
      () => reminders.plan(
        nowUtcMs: any(named: 'nowUtcMs'),
        offsetMs: any(named: 'offsetMs'),
        canNotify: any(named: 'canNotify'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, ReminderPlan>(
        ReminderPlan(showNow: [], schedule: []),
      ),
    );
    when(
      () => reminders.forget(
        keys: any(named: 'keys'),
        nowUtcMs: any(named: 'nowUtcMs'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, void>(null),
    );
    when(
      () => reminders.forgetFuture(nowUtcMs: any(named: 'nowUtcMs')),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, void>(null),
    );
    coordinator = RemindersCoordinator(
      preferences,
      reminders,
      notifier,
      library,
    );
  });

  setUpAll(() => registerFallbackValue((Preferences it) => it));

  Future<RemindersSettingsBloc> started() async {
    final bloc = RemindersSettingsBloc(preferences, notifier, coordinator)
      ..add(const RemindersSettingsEvent.started());
    addTearDown(bloc.close);
    await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    return bloc;
  }

  group('RemindersSettingsBloc', () {
    test('reminders are on until turned off', () async {
      final bloc = await started();
      expect(bloc.state.enabled, isTrue);
      expect(bloc.state.permission, ReminderPermission.notDetermined);
    });

    test('turning it on asks for permission once', () async {
      saved = const Preferences(remindersEnabled: false);
      final bloc = await started();
      bloc.add(const RemindersSettingsEvent.toggled(enabled: true));
      final state = await bloc.stream.firstWhere(
        (s) => s.permission == ReminderPermission.granted,
      );
      expect(state.enabled, isTrue);
      expect(notifier.requests, 1);
      expect(saved.remindersEnabled, isTrue);
      bloc
        ..add(const RemindersSettingsEvent.toggled(enabled: false))
        ..add(const RemindersSettingsEvent.toggled(enabled: true));
      await bloc.stream.firstWhere((s) => s.enabled);
      expect(notifier.requests, 1);
    });

    test('a refusal leaves the switch on and says it is blocked', () async {
      notifier.allow = false;
      final bloc = await started();
      bloc.add(const RemindersSettingsEvent.toggled(enabled: true));
      final state = await bloc.stream.firstWhere(
        (s) => s.permission == ReminderPermission.denied,
      );
      expect(state.blockedBySystem, isTrue);
    });

    test('turning it off saves it and removes the reminders', () async {
      notifier.current = ReminderPermission.granted;
      final bloc = await started();
      bloc.add(const RemindersSettingsEvent.toggled(enabled: false));
      await bloc.stream.firstWhere((s) => !s.enabled);
      await Future<void>.delayed(Duration.zero);
      expect(saved.remindersEnabled, isFalse);
      expect(notifier.cancels, 1);
      expect(notifier.requests, 0);
    });

    test('coming back reads the permission again', () async {
      final bloc = await started();
      notifier.current = ReminderPermission.granted;
      bloc.add(const RemindersSettingsEvent.resumed());
      await bloc.stream.firstWhere(
        (s) => s.permission == ReminderPermission.granted,
      );
    });
  });
}
