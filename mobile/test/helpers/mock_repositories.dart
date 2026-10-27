// Kanso — mock_repositories.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:kanso/core/models/repos/abs/about_repository.dart';
import 'package:kanso/core/models/repos/abs/account_repository.dart';
import 'package:kanso/core/models/repos/abs/account_settings_repository.dart';
import 'package:kanso/core/models/repos/abs/approvals_repository.dart';
import 'package:kanso/core/models/repos/abs/auth_session_repository.dart';
import 'package:kanso/core/models/repos/abs/capture_repository.dart';
import 'package:kanso/core/models/repos/abs/gallery_repository.dart';
import 'package:kanso/core/models/repos/abs/haptics_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/models/repos/abs/licences_repository.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/core/models/repos/abs/shared_notes_repository.dart';
import 'package:kanso/core/models/repos/abs/sync_repository.dart';
import 'package:kanso/core/models/repos/abs/system_share_repository.dart';
import 'package:kanso/core/models/repos/abs/widget_colors_repository.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/editor/models/repos/abs/web_link_repository.dart';
import 'package:kanso/lock/models/repos/abs/device_lock_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockLibraryRepository extends Mock implements LibraryRepository {}

class MockSyncRepository extends Mock implements SyncRepository {}

class MockHapticsRepository extends Mock implements HapticsRepository {}

class MockWebLinkRepository extends Mock implements WebLinkRepository {}

class MockAboutRepository extends Mock implements AboutRepository {}

class MockLicencesRepository extends Mock implements LicencesRepository {}

class MockCaptureRepository extends Mock implements CaptureRepository {}

class MockWidgetColorsRepository extends Mock
    implements WidgetColorsRepository {}

class MockSessionRepository extends Mock implements SessionRepository {}

class MockDeviceLockRepository extends Mock implements DeviceLockRepository {}

class MockGalleryRepository extends Mock implements GalleryRepository {}

class MockSharedNotesRepository extends Mock implements SharedNotesRepository {}

class MockSystemShareRepository extends Mock implements SystemShareRepository {}

class MockAccountRepository extends Mock implements AccountRepository {}

class MockAccountSettingsRepository extends Mock
    implements AccountSettingsRepository {}

class MockApprovalsRepository extends Mock implements ApprovalsRepository {}

class MockAuthSessionRepository extends Mock implements AuthSessionRepository {}

class MockUndoNoticeBloc extends MockBloc<UndoNoticeEvent, UndoNoticeState>
    implements UndoNoticeBloc {}
