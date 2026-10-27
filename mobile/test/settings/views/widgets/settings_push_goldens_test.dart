// Kanso — settings_push_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/router/kanso_sheet_page.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import '../../../helpers/fixture_push_repository.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/settings_blocs.dart';

/// A tablet that is not an iPad, in portrait (device pixel ratio 2).
const Size _androidTabletSize = Size(800, 1280);

/// An account signed in, so the Sync section has the desktop's rows.
const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: 'https://example.invalid',
);

/// The windows the settings are shown in, and the panes each has room for:
/// a phone has the page, a tablet the sheet over its panes.
const List<(String, Size, LayoutClass)> _devices =
    <(String, Size, LayoutClass)>[
      ('phone', phoneSize, LayoutClass.single),
      ('ipad_portrait', tabletPortraitSize, LayoutClass.double),
      ('ipad_landscape', tabletLandscapeSize, LayoutClass.triple),
      ('android_tablet', _androidTabletSize, LayoutClass.double),
    ];

/// Settings as the app shows them for [layout]: a page on a phone, and on
/// a tablet the sheet over what is below.
Widget _settings(SettingsBlocs blocs, LayoutClass layout) {
  if (layout == LayoutClass.single) {
    return blocs.provide(
      const Scaffold(resizeToAvoidBottomInset: false, body: SettingsPane()),
    );
  }
  return Scaffold(
    resizeToAvoidBottomInset: false,
    body: Builder(
      builder: (context) => ColoredBox(color: context.colors.list),
    ),
  );
}

/// Scrolls the settings so the wake switch and its note are in the picture.
Future<void> _toPush(WidgetTester tester) async {
  await tester.scrollUntilVisible(
    find.text('Wake this device when notes change elsewhere'),
    200,
    scrollable: find.byType(Scrollable).last,
  );
  await tester.drag(find.byType(Scrollable).last, const Offset(0, 100));
  await tester.pumpAndSettle();
}

/// Opens the sheet of settings over the page, as the app's own route does;
/// the route is above the page, so the appearance and the [blocs] are given
/// to it again.
Future<void> _openSheet(WidgetTester tester, SettingsBlocs blocs) async {
  final context = tester.element(find.byType(Scaffold));
  final route = KansoSheetRoute<void>(
    page: KansoSheetPage<void>(
      child: blocs.provide(
        BlocProvider<AppearanceBloc>.value(
          value: context.read<AppearanceBloc>(),
          child: const SettingsPane(),
        ),
      ),
    ),
  );
  unawaited(Navigator.of(context).push(route));
  await tester.pumpAndSettle();
  await _toPush(tester);
}

void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the wake switch in settings, $name', () {
      for (final (device, size, layout) in _devices) {
        testWidgets('on $device', (tester) async {
          expect(KansoLayout.classFor(size), layout);
          final blocs = await SettingsBlocs.start(
            account: _signedIn,
            status: signedInStatus,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            _settings(blocs, layout),
            name: '${device}_settings_push_$name',
            size: size,
            brightness: brightness,
            interact: layout == LayoutClass.single
                ? _toPush
                : (tester) => _openSheet(tester, blocs),
          );
        });
      }

      testWidgets('turned off on a phone', (tester) async {
        final blocs = await SettingsBlocs.start(
          account: _signedIn,
          status: signedInStatus,
          push: FixturePushRepository(enabled: false),
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _settings(blocs, LayoutClass.single),
          name: 'phone_settings_push_off_$name',
          size: phoneSize,
          brightness: brightness,
          interact: _toPush,
        );
      });

      testWidgets('where push cannot work, on a phone', (tester) async {
        final blocs = await SettingsBlocs.start(
          account: _signedIn,
          status: signedInStatus,
          push: FixturePushRepository(unavailable: 'No Firebase.'),
        );
        await expectGoldenOnBothPlatforms(
          tester,
          _settings(blocs, LayoutClass.single),
          name: 'phone_settings_push_unavailable_$name',
          size: phoneSize,
          brightness: brightness,
          interact: _toPush,
        );
      });
    });
  }
}
