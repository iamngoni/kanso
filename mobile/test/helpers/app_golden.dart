// Kanso — app_golden.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/import/state/opened_documents/opened_documents_bloc.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';

import 'account_blocs.dart';
import 'browse_blocs.dart';
import 'fixture_device_lock_repository.dart';
import 'fixture_gallery_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_parity_repositories.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_session_repository.dart';
import 'fixture_shared_notes_repository.dart';
import 'fixture_sync_repository.dart';
import 'golden.dart';
import 'kanso_viewport.dart';
import 'pump_browse_app.dart';

/// Takes a picture of the whole app, as a device of [viewport] draws it, once
/// as iOS and once as Android, as [expectGoldenOnBothPlatforms] does for a
/// widget.
///
/// The app is the real one over the mock library (see [pumpBrowseApp]): its
/// router, shell and panes, so the picture is what the layout class of the
/// window makes of the screen and not a pane on its own. It opens at [at];
/// [interact] then runs, once per platform, on the settled app, to open a
/// sheet or a note, or to start a selection.
///
/// [library] and [renderer] make the mock library and the renderer for each
/// platform, so a test can fail or hold back what they answer; [appearance]
/// is what the app starts showing (a theme, an icon theme); [accounts] makes
/// the account screens' BLoCs for each platform; [backupNotice] makes the
/// backup notice's BLoC; [shared] and [gallery] make the shared notes and
/// the theme gallery the app reads; [session] and [device] make the lock
/// session and the phone's biometrics. The clock is the mock content's (see
/// [atMockNow]).
Future<void> expectAppGoldenOnBothPlatforms(
  WidgetTester tester, {
  required String name,
  required KansoViewport viewport,
  required Brightness brightness,
  String at = '/notes',
  FixtureLibraryRepository Function()? library,
  FixtureRendererRepository Function()? renderer,
  AppearanceState appearance = const AppearanceState(),
  Future<BackupNoticeBloc> Function()? backupNotice,
  FixtureSharedNotesRepository Function()? shared,
  FixtureGalleryRepository Function()? gallery,
  AccountBlocs Function()? accounts,
  FixtureSyncRepository Function()? sync,
  FixtureSessionRepository Function()? session,
  FixtureDeviceLockRepository Function()? device,
  OpenedDocumentsBloc? openedDocuments,
  FixtureHistoryRepository Function()? history,
  FixtureReplaceRepository Function()? replace,
  FixtureBackupsRepository Function()? backups,
  Future<void> Function(WidgetTester tester, BrowseApp app)? interact,
}) async {
  tester.platformDispatcher
    ..textScaleFactorTestValue = 1
    ..platformBrightnessTestValue = brightness
    ..accessibilityFeaturesTestValue = const FakeAccessibilityFeatures(
      disableAnimations: true,
      reduceMotion: true,
    );
  addTearDown(() {
    tester.view.reset();
    tester.platformDispatcher
      ..clearTextScaleFactorTestValue()
      ..clearPlatformBrightnessTestValue()
      ..clearAccessibilityFeaturesTestValue();
  });

  final renders = <TargetPlatform, Uint8List>{};
  for (final platform in <TargetPlatform>[
    TargetPlatform.iOS,
    TargetPlatform.android,
  ]) {
    debugDefaultTargetPlatformOverride = platform;
    debugDisableShadows = false;
    try {
      await atMockNow(() async {
        final app = await pumpBrowseApp(
          tester,
          viewport.size,
          at: at,
          library: library?.call(),
          renderer: renderer?.call(),
          appearance: appearance,
          devicePixelRatio: viewport.devicePixelRatio,
          safeArea: viewport.safeArea,
          tabletDevice: viewport.tablet,
          backupNotice: backupNotice == null ? null : await backupNotice(),
          shared: shared?.call(),
          gallery: gallery?.call(),
          accounts: accounts?.call(),
          sync: sync?.call(),
          session: session?.call(),
          device: device?.call(),
          openedDocuments: openedDocuments,
          history: history?.call(),
          replace: replace?.call(),
          backups: backups?.call(),
        );
        if (interact != null) {
          await interact(tester, app);
          await tester.pumpAndSettle();
        }
      });
      final boundary = tester.renderObject<RenderRepaintBoundary>(
        find.byKey(appBoundaryKey),
      );
      renders[platform] = (await tester.runAsync(() async {
        final image = await boundary.toImage(
          pixelRatio: viewport.devicePixelRatio,
        );
        final data = await image.toByteData();
        image.dispose();
        return data!.buffer.asUint8List();
      }))!;
      if (platform == TargetPlatform.iOS) {
        await expectLater(
          find.byKey(appBoundaryKey),
          matchesGoldenFile('goldens/$name.png'),
        );
      }
      // The next platform starts from a window with nothing in it.
      await tester.pumpWidget(const SizedBox.shrink());
    } finally {
      debugDisableShadows = true;
      debugDefaultTargetPlatformOverride = null;
    }
  }

  final differing = differingPixels(
    renders[TargetPlatform.iOS]!,
    renders[TargetPlatform.android]!,
  );
  expect(
    differing,
    0,
    reason: '$differing pixels differ between the iOS and Android renders',
  );
}
