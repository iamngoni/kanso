// Kanso — plugin_push_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: only_throw_errors

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/sync.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/push/models/data/push_token.dart';
import 'package:kanso/push/models/repos/impl/plugin_push_repository.dart';
import 'package:kanso_push/kanso_push.dart';

/// The native side of the plugin, as a test has it.
class FakePushHost extends Fake implements KansoPushHostApi {
  String? reason;
  Object? failure;
  int registers = 0;

  @override
  Future<void> register() async {
    registers++;
    final error = failure;
    if (error != null) {
      throw error;
    }
  }

  @override
  Future<String?> unavailableReason() async => reason;
}

/// The Rust facade, recording what the repository hands it.
class FakeSync extends Fake implements KansoSync {
  final List<PushToken> tokens = <PushToken>[];
  final List<bool> enabledCalls = <bool>[];
  int refreshes = 0;
  bool enabled = true;
  Object? failure;

  @override
  Future<void> setPushToken({
    required PushPlatformKind platform,
    required String token,
    required PushEnvironmentKind environment,
  }) async {
    final error = failure;
    if (error != null) {
      throw error;
    }
    tokens.add(
      PushToken(platform: platform, token: token, environment: environment),
    );
  }

  @override
  Future<void> refreshPush() async {
    final error = failure;
    if (error != null) {
      throw error;
    }
    refreshes++;
  }

  @override
  Future<bool> pushEnabled() async => enabled;

  @override
  Future<void> setPushEnabled({required bool enabled}) async {
    final error = failure;
    if (error != null) {
      throw error;
    }
    enabledCalls.add(enabled);
    this.enabled = enabled;
  }
}

void main() {
  late FakePushHost host;
  late FakeSync sync;
  late PluginPushRepository repository;

  setUp(() {
    host = FakePushHost();
    sync = FakeSync();
    repository = PluginPushRepository(host, sync);
    addTearDown(() => KansoPushFlutterApi.setUp(null));
  });

  group('PluginPushRepository', () {
    test('says push works when the plugin has no reason it cannot', () async {
      expect(
        await repository.unavailableReason(),
        const Right<ApplicationError, String?>(null),
      );
    });

    test('says why push cannot work when the plugin does', () async {
      host.reason = 'This build has no Firebase configuration.';
      expect(
        await repository.unavailableReason(),
        const Right<ApplicationError, String?>(
          'This build has no Firebase configuration.',
        ),
      );
    });

    test('a plugin that is not there is an error, not a crash', () async {
      host.failure = MissingPluginException('no kanso_push');
      final result = await repository.register();
      expect(result.swap().toOption().toNullable(), isA<UnexpectedError>());
    });

    test('registering asks the OS', () async {
      expect(
        await repository.register(),
        const Right<ApplicationError, Unit>(unit),
      );
      expect(host.registers, 1);
    });

    test('a token from the plugin, after registering, is heard', () async {
      await repository.register();
      final heard = expectLater(
        repository.tokens(),
        emits(
          const PushToken(
            platform: PushPlatformKind.apns,
            token: 'abcdef',
            environment: PushEnvironmentKind.development,
          ),
        ),
      );
      repository.onToken('apns', 'abcdef', 'development');
      await heard;
    });

    test("each service and environment becomes the library's", () async {
      final heard = <PushToken>[];
      final subscription = repository.tokens().listen(heard.add);
      addTearDown(subscription.cancel);
      repository
        ..onToken('fcm', 't1', 'production')
        ..onToken('apns', 't2', 'production')
        ..onToken('apns', 't3', 'development');
      await pumpEventQueue();
      expect(
        heard.map((t) => (t.platform, t.environment)),
        <(PushPlatformKind, PushEnvironmentKind)>[
          (PushPlatformKind.fcm, PushEnvironmentKind.production),
          (PushPlatformKind.apns, PushEnvironmentKind.production),
          (PushPlatformKind.apns, PushEnvironmentKind.development),
        ],
      );
    });

    test(
      'a service, an environment or a token it does not know is dropped',
      () async {
        final heard = <PushToken>[];
        final subscription = repository.tokens().listen(heard.add);
        addTearDown(subscription.cancel);
        repository
          ..onToken('web', 't1', 'production')
          ..onToken('fcm', 't2', 'staging')
          ..onToken('fcm', '', 'production');
        await pumpEventQueue();
        expect(heard, isEmpty);
      },
    );

    test('a token goes to the library as it is', () async {
      const token = PushToken(
        platform: PushPlatformKind.fcm,
        token: 'fcm-token',
        environment: PushEnvironmentKind.production,
      );
      expect(
        await repository.setToken(token),
        const Right<ApplicationError, Unit>(unit),
      );
      expect(sync.tokens, <PushToken>[token]);
    });

    test('a token the server would refuse is an invalid error', () async {
      sync.failure = const KansoError.invalid(message: 'Not a token.');
      final result = await repository.setToken(
        const PushToken(
          platform: PushPlatformKind.fcm,
          token: 'x',
          environment: PushEnvironmentKind.production,
        ),
      );
      expect(result.swap().toOption().toNullable(), isA<InvalidError>());
    });

    test('a server that cannot be reached is a network error', () async {
      sync.failure = const KansoError.network(message: 'Offline.');
      expect(
        (await repository.refresh()).swap().toOption().toNullable(),
        isA<NetworkError>(),
      );
      expect(
        (await repository.setEnabled(
          enabled: false,
        )).swap().toOption().toNullable(),
        isA<NetworkError>(),
      );
    });

    test('refreshing has the library bring the server in line', () async {
      await repository.refresh();
      expect(sync.refreshes, 1);
    });

    test('the setting is read from and written to the library', () async {
      expect(
        await repository.isEnabled(),
        const Right<ApplicationError, bool>(true),
      );
      await repository.setEnabled(enabled: false);
      expect(sync.enabledCalls, <bool>[false]);
      expect(
        await repository.isEnabled(),
        const Right<ApplicationError, bool>(false),
      );
    });

    test('a token is not printed', () {
      const token = PushToken(
        platform: PushPlatformKind.fcm,
        token: 'secret-token-value',
        environment: PushEnvironmentKind.production,
      );
      expect('$token', isNot(contains('secret-token-value')));
    });
  });
}
