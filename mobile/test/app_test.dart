// Kanso — app_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/app.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/core/configs/kanso_theme_scope.dart';
import 'package:kanso/core/configs/kanso_typography.dart';
import 'package:kanso/core/models/data/design_gallery_access.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/core/router/layout_class_notifier.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/device_approval/device_approval_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_feedback.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/design/views/pages/design_gallery_page.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/settings/state/sync_settings/sync_settings_bloc.dart';

import 'helpers/account_blocs.dart';
import 'helpers/app_feature_fixtures.dart';
import 'helpers/close_bloc_later.dart';
import 'helpers/crash_samples.dart';
import 'helpers/fake_canvas.dart';
import 'helpers/fixture_sync_repository.dart';
import 'helpers/seeded_appearance_bloc.dart';
import 'helpers/sync_settings_blocs.dart';

Future<AppearanceBloc> pumpApp(
  WidgetTester tester, {
  AppearanceState state = const AppearanceState(),
}) async {
  final bloc = SeededAppearanceBloc(state);
  addTearDown(bloc.close);
  tester.view
    ..physicalSize = const Size(393, 2400)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final accounts = AccountBlocs();
  final appFeatures = AppFeatureFixtures()..register();
  getIt.registerSingleton<CanvasRepository>(FakeCanvasRepository());
  addTearDown(appFeatures.dispose);
  addTearDown(() async => getIt.reset());
  final sync = SyncStatusBloc(FixtureSyncRepository());
  closeBlocLater(sync);
  final router = KansoRouter(
    const DesignGalleryAccess(available: true),
    LayoutClassNotifier.starting(LayoutClass.single),
  )..config.go(KansoRoutes.design);
  await tester.pumpWidget(
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: bloc),
        BlocProvider<AiStatusBloc>.value(value: appFeatures.status),
        BlocProvider<BackupNoticeBloc>.value(value: hiddenBackupNoticeBloc()),
        BlocProvider<SyncStatusBloc>.value(value: sync),
        BlocProvider<SyncSettingsBloc>.value(value: startedSyncSettingsBloc()),
        BlocProvider<AccountBloc>.value(value: accounts.account),
        BlocProvider<DeviceApprovalBloc>.value(value: accounts.approval),
        BlocProvider<CrashReportsBloc>.value(value: quietCrashReportsBloc()),
      ],
      child: App(router: router),
    ),
  );
  return bloc;
}

void main() {
  Future<KansoTypography> typography(WidgetTester tester) async {
    await tester.pumpAndSettle();
    return KansoThemeScope.of(
      tester.element(find.byType(DesignGalleryPage)),
    ).typography;
  }

  group('App', () {
    testWidgets('applies downloaded Google Font families to the note theme', (
      tester,
    ) async {
      await pumpApp(
        tester,
        state: const AppearanceState(
          fonts: KansoFonts(
            editorGoogleFont: 'Lora',
            previewGoogleFont: 'Nunito Sans',
          ),
        ),
      );
      final styles = await typography(tester);
      expect(styles.editorBody.fontFamily, 'Lora');
      expect(styles.previewBody.fontFamily, 'Nunito Sans');
    });

    testWidgets('builds its themes from the fonts and text size chosen', (
      tester,
    ) async {
      final bloc = await pumpApp(tester);
      var styles = await typography(tester);
      expect(styles.previewBody.fontFamily, 'Instrument Sans');
      expect(styles.previewBody.fontSize, 18);

      bloc
        ..add(
          const AppearanceEvent.previewFontChanged(
            KansoFontFamily.sourceSerif4,
          ),
        )
        ..add(const AppearanceEvent.textSizeChanged(1.15));
      styles = await typography(tester);
      expect(styles.previewBody.fontFamily, 'Source Serif 4');
      expect(styles.previewBody.fontSize, 18);
      expect(styles.editorBody.fontSize, closeTo(18 * 1.15, 1e-9));
      bloc.add(
        const AppearanceEvent.previewTextSizeChanged(1.3),
      );
      styles = await typography(tester);
      expect(styles.previewBody.fontSize, closeTo(18 * 1.3, 1e-9));
      // The interface is Kanso's own, whatever the notes are set in.
      expect(styles.body.fontFamily, 'Instrument Sans');
      expect(styles.body.fontSize, 16);
    });

    testWidgets('scales the interface without changing note text', (
      tester,
    ) async {
      final bloc = await pumpApp(tester);
      var styles = await typography(tester);
      expect(styles.body.fontSize, 16);
      expect(styles.editorBody.fontSize, 18);
      bloc.add(const AppearanceEvent.interfaceTextSizeChanged(1.2));
      styles = await typography(tester);
      expect(styles.body.fontSize, closeTo(19.2, 1e-9));
      expect(styles.barTitle.fontSize, closeTo(21.6, 1e-9));
      expect(styles.editorBody.fontSize, 18);
      expect(styles.previewBody.fontSize, 18);
    });

    testWidgets('the dark theme follows the same choices', (tester) async {
      final bloc = await pumpApp(tester);
      bloc.add(
        const AppearanceEvent.codeFontChanged(KansoFontFamily.ibmPlexMono),
      );
      await tester.pumpAndSettle();
      final app = tester.widget<WidgetsApp>(find.byType(WidgetsApp));
      expect(
        KansoThemeScope.of(
          tester.element(find.byType(DesignGalleryPage)),
        ).typography.mono.fontFamily,
        'IBM Plex Mono',
      );
      expect(app.debugShowCheckedModeBanner, isFalse);
      expect(find.byType(KansoFeedbackHost), findsOneWidget);
    });

    testWidgets("does not clamp the device's text scale", (tester) async {
      tester.platformDispatcher.textScaleFactorTestValue = 2.5;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      await pumpApp(tester);
      await tester.pumpAndSettle();
      final scale = MediaQuery.textScalerOf(
        tester.element(find.byType(DesignGalleryPage)),
      ).scale(10);
      expect(scale, 25);
    });
  });
}
