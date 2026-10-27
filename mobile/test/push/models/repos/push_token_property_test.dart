// Kanso — push_token_property_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, CombinableAny, Generator, Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/push/models/data/push_token.dart';
import 'package:kanso/push/models/repos/impl/plugin_push_repository.dart';
import 'package:kanso/push/state/push_registration/push_registration_bloc.dart';

import '../../../helpers/property.dart';
import 'plugin_push_repository_test.dart';

/// What the plugin may call `onToken` with: the services and environments it
/// has, and a few it doesn't, which are dropped.
typedef TokenEvent = (String platform, String token, String environment);

const List<String> _platforms = <String>['apns', 'fcm', 'web'];
const List<String> _environments = <String>['production', 'development', 'x'];

extension on Any {
  Generator<TokenEvent> get tokenEvent => combine3(
    intInRange(0, _platforms.length),
    intInRange(0, 1000),
    intInRange(0, _environments.length),
    (int platform, int token, int environment) =>
        (_platforms[platform], 'token-$token', _environments[environment]),
  );
}

PushToken? _known(TokenEvent event) {
  final platform = switch (event.$1) {
    'apns' => PushPlatformKind.apns,
    'fcm' => PushPlatformKind.fcm,
    _ => null,
  };
  final environment = switch (event.$3) {
    'production' => PushEnvironmentKind.production,
    'development' => PushEnvironmentKind.development,
    _ => null,
  };
  if (platform == null || environment == null) {
    return null;
  }
  return PushToken(
    platform: platform,
    token: event.$2,
    environment: environment,
  );
}

/// Whatever tokens the OS issues, in whatever order, with the platform and
/// environment it says, the library is handed every known one in the order
/// they came, so the last token is the one it keeps, with the right
/// platform and environment.
void main() {
  Glados<List<TokenEvent>>(
    any.listWithLengthInRange(1, 12, any.tokenEvent),
    kansoExploreConfig(),
  ).test('the library is handed the last token, with its service and '
      'environment', (events) async {
    final sync = FakeSync();
    final repository = PluginPushRepository(FakePushHost(), sync);
    final bloc = PushRegistrationBloc(repository)
      ..add(const PushRegistrationEvent.started());
    await pumpEventQueue();
    for (final event in events) {
      repository.onToken(event.$1, event.$2, event.$3);
    }
    await pumpEventQueue();
    await bloc.close();

    final expected = events.map(_known).whereType<PushToken>().toList();
    expect(sync.tokens, expected);
    if (expected.isNotEmpty) {
      expect(sync.tokens.last, expected.last);
    }
  });
}
