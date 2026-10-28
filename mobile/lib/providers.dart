// Kanso — providers.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'backups/state/backups/backups_bloc.dart';
import 'core/state/account/account_bloc.dart';
import 'core/state/appearance/appearance_bloc.dart';
import 'core/state/capture_inbox/capture_inbox_bloc.dart';
import 'core/state/device_approval/device_approval_bloc.dart';
import 'core/state/library_index/library_index_bloc.dart';
import 'core/state/list_navigation/list_navigation_bloc.dart';
import 'core/state/navigation_history/navigation_history_bloc.dart';
import 'core/state/sync_status/sync_status_bloc.dart';
import 'core/state/undo_notice/undo_notice_bloc.dart';
import 'core/state/widget_colors/widget_colors_bloc.dart';
import 'core/views/widgets/capture_inbox_host.dart';
import 'core/views/widgets/widget_colors_host.dart';
import 'crash/state/crash_reports/crash_reports_bloc.dart';
import 'import/state/opened_documents/opened_documents_bloc.dart';
import 'injection.dart';
import 'lock/state/lock_status/lock_status_bloc.dart';
import 'lock/state/privacy_cover/privacy_cover_bloc.dart';
import 'settings/state/backup_notice/backup_notice_bloc.dart';
import 'settings/state/default_notebook/default_notebook_bloc.dart';
import 'settings/state/sync_settings/sync_settings_bloc.dart';
import 'sharing/state/invitations/invitations_bloc.dart';
import 'shell/state/library_pane/library_pane_bloc.dart';
import 'shell/state/note_creation/note_creation_bloc.dart';

/// The BLoCs the whole app shares, above the [child].
///
/// [appearance] is read before the app runs (see `bootstrap`), so the first
/// frame is already in the chosen themes.
class AppProviders extends StatelessWidget {
  const AppProviders({
    required this.appearance,
    required this.child,
    super.key,
  });

  final AppearanceBloc appearance;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<BackupsBloc>(
          create: (_) =>
              getIt<BackupsBloc>()..add(const BackupsEvent.started()),
        ),
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<LibraryIndexBloc>(
          create: (_) =>
              getIt<LibraryIndexBloc>()..add(const LibraryIndexEvent.started()),
        ),
        BlocProvider<AccountBloc>(
          create: (_) =>
              getIt<AccountBloc>()..add(const AccountEvent.started()),
        ),
        BlocProvider<DeviceApprovalBloc>(
          create: (_) => getIt<DeviceApprovalBloc>(),
        ),
        BlocProvider<SyncStatusBloc>(
          create: (_) =>
              getIt<SyncStatusBloc>()..add(const SyncStatusEvent.started()),
        ),
        BlocProvider<LockStatusBloc>(
          create: (_) =>
              getIt<LockStatusBloc>()..add(const LockStatusEvent.started()),
        ),
        BlocProvider<PrivacyCoverBloc>(
          create: (_) => getIt<PrivacyCoverBloc>(),
        ),
        BlocProvider<BackupNoticeBloc>(
          create: (_) =>
              getIt<BackupNoticeBloc>()..add(const BackupNoticeEvent.started()),
        ),
        BlocProvider<InvitationsBloc>(
          create: (_) =>
              getIt<InvitationsBloc>()..add(const InvitationsEvent.started()),
        ),
        BlocProvider<DefaultNotebookBloc>(
          create: (_) =>
              getIt<DefaultNotebookBloc>()
                ..add(const DefaultNotebookEvent.started()),
        ),
        BlocProvider<SyncSettingsBloc>(
          create: (_) =>
              getIt<SyncSettingsBloc>()..add(const SyncSettingsEvent.started()),
        ),
        BlocProvider<CrashReportsBloc>(
          create: (_) =>
              getIt<CrashReportsBloc>()..add(const CrashReportsEvent.started()),
        ),
        BlocProvider<OpenedDocumentsBloc>(
          create: (_) =>
              getIt<OpenedDocumentsBloc>()
                ..add(const OpenedDocumentsEvent.started()),
        ),
        BlocProvider<ListNavigationBloc>(
          create: (_) => getIt<ListNavigationBloc>(),
        ),
        BlocProvider<LibraryPaneBloc>(create: (_) => getIt<LibraryPaneBloc>()),
        BlocProvider<NoteCreationBloc>(
          create: (_) => getIt<NoteCreationBloc>(),
        ),
        BlocProvider<NavigationHistoryBloc>(
          create: (_) => getIt<NavigationHistoryBloc>(),
        ),
        BlocProvider<CaptureInboxBloc>(
          create: (_) =>
              getIt<CaptureInboxBloc>()
                ..add(const CaptureInboxEvent.saveRequested()),
        ),
        BlocProvider<WidgetColorsBloc>(
          create: (_) => getIt<WidgetColorsBloc>()
            ..add(
              WidgetColorsEvent.changed(
                light: appearance.state.light.colors,
                dark: appearance.state.dark.colors,
              ),
            ),
        ),
        // The note list shows its notices through the same one.
        BlocProvider<UndoNoticeBloc>.value(value: getIt<UndoNoticeBloc>()),
      ],
      // What other apps shared is saved as the app starts (above) and each
      // time it comes back to the front; the widgets follow the themes.
      child: CaptureInboxHost(child: WidgetColorsHost(child: child)),
    );
  }
}
