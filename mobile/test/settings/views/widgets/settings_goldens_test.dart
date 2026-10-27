// Kanso — settings_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/views/widgets/backup_notice_host.dart';
import 'package:kanso/settings/views/widgets/fonts_pane.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/settings_blocs.dart';

Widget _page(Widget child) {
  return Scaffold(resizeToAvoidBottomInset: false, body: child);
}

void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the settings pictures, $name', () {
      testWidgets('settings', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          (await SettingsBlocs.start()).provide(_page(const SettingsPane())),
          name: 'phone_settings_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('settings with a sync account', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          (await SettingsBlocs.start(
            account: const AccountState.signedIn(
              username: 'ana',
              server: 'https://sync.kansoapp.app',
            ),
            status: const SyncStatus(
              phase: SyncPhase.synced,
              username: 'ana',
              conflicts: 0,
              skippedCount: 0,
              account: SyncAccount.signedIn(),
            ),
          )).provide(_page(const SettingsPane())),
          name: 'phone_settings_synced_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('fonts', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          _page(const FontsPane()),
          name: 'phone_fonts_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('fonts, at the text size', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          _page(const FontsPane(startAtEnd: true)),
          name: 'phone_fonts_text_size_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the backup notice over the list', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final notice = BackupNoticeBloc(ForgetfulPreferencesRepository())
          ..add(const BackupNoticeEvent.started());
        await notice.stream.firstWhere((state) => state.visible);
        closeBlocLater(notice);
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            BlocProvider<BackupNoticeBloc>.value(
              value: notice,
              child: blocs.provide(
                BackupNoticeHost(child: _page(const NoteListPane())),
                list: list,
              ),
            ),
            name: 'phone_backup_notice_$name',
            size: phoneSize,
            brightness: brightness,
          ),
        );
      });
    });
  }
}
