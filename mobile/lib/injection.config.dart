// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

// **************************************************************************
// InjectableConfigGenerator
// **************************************************************************

// ignore_for_file: type=lint
// coverage:ignore-file

// ignore_for_file: no_leading_underscores_for_library_prefixes

import 'package:get_it/get_it.dart' as _i174;
import 'package:image_picker/image_picker.dart' as _i183;
import 'package:injectable/injectable.dart' as _i526;
import 'package:kanso_push/kanso_push.dart' as _i901;

import 'account/state/account_details/account_details_bloc.dart' as _i645;
import 'account/state/recover/recover_bloc.dart' as _i1059;
import 'account/state/recovery_key/recovery_key_bloc.dart' as _i35;
import 'account/state/sign_in/sign_in_bloc.dart' as _i552;
import 'account/state/unlock/unlock_bloc.dart' as _i858;
import 'backups/models/repos/abs/backups_repository.dart' as _i1002;
import 'backups/models/repos/impl/rust_backups_repository.dart' as _i270;
import 'backups/state/backups/backups_bloc.dart' as _i770;
import 'board/state/board/board_bloc.dart' as _i553;
import 'capture/models/repos/abs/media_picker_repository.dart' as _i900;
import 'capture/models/repos/impl/platform_media_picker_repository.dart'
    as _i598;
import 'capture/state/daily_note/daily_note_bloc.dart' as _i1060;
import 'core/bridge/api/account.dart' as _i578;
import 'core/bridge/api/account_settings.dart' as _i102;
import 'core/bridge/api/approvals.dart' as _i283;
import 'core/bridge/api/gallery.dart' as _i783;
import 'core/bridge/api/library.dart' as _i455;
import 'core/bridge/api/locks.dart' as _i983;
import 'core/bridge/api/renderer.dart' as _i832;
import 'core/bridge/api/shared.dart' as _i315;
import 'core/bridge/api/sync.dart' as _i591;
import 'core/configs/app_module.dart' as _i216;
import 'core/configs/core_module.dart' as _i418;
import 'core/models/data/design_gallery_access.dart' as _i289;
import 'core/models/data/library_folder.dart' as _i72;
import 'core/models/repos/abs/about_repository.dart' as _i80;
import 'core/models/repos/abs/account_repository.dart' as _i1035;
import 'core/models/repos/abs/account_settings_repository.dart' as _i314;
import 'core/models/repos/abs/approvals_repository.dart' as _i465;
import 'core/models/repos/abs/auth_session_repository.dart' as _i227;
import 'core/models/repos/abs/board_repository.dart' as _i235;
import 'core/models/repos/abs/capture_repository.dart' as _i23;
import 'core/models/repos/abs/diagnostics_repository.dart' as _i984;
import 'core/models/repos/abs/gallery_repository.dart' as _i550;
import 'core/models/repos/abs/haptics_repository.dart' as _i1056;
import 'core/models/repos/abs/journal_repository.dart' as _i169;
import 'core/models/repos/abs/library_repository.dart' as _i38;
import 'core/models/repos/abs/licences_repository.dart' as _i951;
import 'core/models/repos/abs/patch_repository.dart' as _i909;
import 'core/models/repos/abs/preferences_repository.dart' as _i384;
import 'core/models/repos/abs/reminder_notifier.dart' as _i874;
import 'core/models/repos/abs/reminders_repository.dart' as _i58;
import 'core/models/repos/abs/renderer_repository.dart' as _i732;
import 'core/models/repos/abs/session_repository.dart' as _i333;
import 'core/models/repos/abs/shared_notes_repository.dart' as _i374;
import 'core/models/repos/abs/sync_repository.dart' as _i601;
import 'core/models/repos/abs/system_share_repository.dart' as _i202;
import 'core/models/repos/abs/templates_repository.dart' as _i460;
import 'core/models/repos/abs/widget_colors_repository.dart' as _i978;
import 'core/models/repos/impl/channel_reminder_notifier.dart' as _i740;
import 'core/models/repos/impl/file_preferences_repository.dart' as _i207;
import 'core/models/repos/impl/file_widget_colors_repository.dart' as _i2;
import 'core/models/repos/impl/flutter_haptics_repository.dart' as _i438;
import 'core/models/repos/impl/flutter_licences_repository.dart' as _i1025;
import 'core/models/repos/impl/flutter_system_share_repository.dart' as _i254;
import 'core/models/repos/impl/package_info_about_repository.dart' as _i489;
import 'core/models/repos/impl/plugin_auth_session_repository.dart' as _i680;
import 'core/models/repos/impl/rust_account_repository.dart' as _i450;
import 'core/models/repos/impl/rust_account_settings_repository.dart' as _i350;
import 'core/models/repos/impl/rust_approvals_repository.dart' as _i99;
import 'core/models/repos/impl/rust_board_repository.dart' as _i418;
import 'core/models/repos/impl/rust_capture_repository.dart' as _i824;
import 'core/models/repos/impl/rust_diagnostics_repository.dart' as _i1062;
import 'core/models/repos/impl/rust_gallery_repository.dart' as _i692;
import 'core/models/repos/impl/rust_journal_repository.dart' as _i212;
import 'core/models/repos/impl/rust_library_repository.dart' as _i152;
import 'core/models/repos/impl/rust_reminders_repository.dart' as _i429;
import 'core/models/repos/impl/rust_renderer_repository.dart' as _i820;
import 'core/models/repos/impl/rust_session_repository.dart' as _i899;
import 'core/models/repos/impl/rust_shared_notes_repository.dart' as _i628;
import 'core/models/repos/impl/rust_sync_repository.dart' as _i564;
import 'core/models/repos/impl/rust_templates_repository.dart' as _i213;
import 'core/models/repos/impl/shorebird_patch_repository.dart' as _i989;
import 'core/router/kanso_router.dart' as _i388;
import 'core/router/layout_class_notifier.dart' as _i609;
import 'core/services/external_changes.dart' as _i825;
import 'core/services/library_folder_migration.dart' as _i846;
import 'core/services/widget_reloader.dart' as _i608;
import 'core/state/account/account_bloc.dart' as _i168;
import 'core/state/appearance/appearance_bloc.dart' as _i551;
import 'core/state/capture_inbox/capture_inbox_bloc.dart' as _i551;
import 'core/state/device_approval/device_approval_bloc.dart' as _i274;
import 'core/state/library_index/library_index_bloc.dart' as _i861;
import 'core/state/list_navigation/list_navigation_bloc.dart' as _i907;
import 'core/state/navigation_history/navigation_history_bloc.dart' as _i272;
import 'core/state/sync_status/sync_status_bloc.dart' as _i185;
import 'core/state/undo_notice/undo_notice_bloc.dart' as _i985;
import 'core/state/widget_colors/widget_colors_bloc.dart' as _i223;
import 'crash/configs/crash_module.dart' as _i1016;
import 'crash/models/data/crash_folder.dart' as _i392;
import 'crash/models/repos/abs/crash_context_repository.dart' as _i1070;
import 'crash/models/repos/abs/crash_report_repository.dart' as _i288;
import 'crash/models/repos/impl/platform_crash_context_repository.dart'
    as _i756;
import 'crash/state/crash_reports/crash_reports_bloc.dart' as _i815;
import 'discovery/models/repos/abs/discovery_repository.dart' as _i673;
import 'discovery/models/repos/impl/rust_discovery_repository.dart' as _i913;
import 'editor/models/repos/abs/editor_repository.dart' as _i979;
import 'editor/models/repos/abs/web_link_repository.dart' as _i272;
import 'editor/models/repos/impl/rust_editor_repository.dart' as _i606;
import 'editor/models/repos/impl/url_launcher_web_link_repository.dart'
    as _i635;
import 'editor/state/editor/editor_bloc.dart' as _i964;
import 'graph/state/graph/graph_bloc.dart' as _i365;
import 'history/models/repos/abs/history_repository.dart' as _i974;
import 'history/models/repos/impl/rust_history_repository.dart' as _i1065;
import 'history/state/note_history/note_history_bloc.dart' as _i403;
import 'import/models/repos/abs/import_files_repository.dart' as _i26;
import 'import/models/repos/abs/import_repository.dart' as _i242;
import 'import/models/repos/impl/channel_import_files_repository.dart' as _i196;
import 'import/models/repos/impl/rust_import_repository.dart' as _i903;
import 'import/state/import_flow/import_flow_bloc.dart' as _i513;
import 'import/state/opened_documents/opened_documents_bloc.dart' as _i979;
import 'journal/state/calendar/calendar_bloc.dart' as _i390;
import 'journal/state/journal/journal_bloc.dart' as _i686;
import 'library/state/new_notebook/new_notebook_bloc.dart' as _i991;
import 'lock/models/repos/abs/device_lock_repository.dart' as _i279;
import 'lock/models/repos/impl/plugin_device_lock_repository.dart' as _i64;
import 'lock/state/biometrics/biometrics_bloc.dart' as _i477;
import 'lock/state/change_password/change_password_bloc.dart' as _i39;
import 'lock/state/lock_remove/lock_remove_bloc.dart' as _i215;
import 'lock/state/lock_setup/lock_setup_bloc.dart' as _i249;
import 'lock/state/lock_status/lock_status_bloc.dart' as _i57;
import 'lock/state/privacy_cover/privacy_cover_bloc.dart' as _i330;
import 'lock/state/unlock/unlock_bloc.dart' as _i797;
import 'note/models/repos/abs/note_place_repository.dart' as _i930;
import 'note/models/repos/impl/memory_note_place_repository.dart' as _i503;
import 'note/state/note/note_bloc.dart' as _i618;
import 'note/state/note_discovery/note_discovery_bloc.dart' as _i779;
import 'note/state/preview/preview_bloc.dart' as _i435;
import 'note_list/state/note_list/note_list_bloc.dart' as _i351;
import 'palette/models/repos/abs/palette_repository.dart' as _i704;
import 'palette/models/repos/impl/rust_palette_repository.dart' as _i97;
import 'palette/state/palette/palette_bloc.dart' as _i204;
import 'push/configs/push_module.dart' as _i78;
import 'push/models/repos/abs/push_repository.dart' as _i937;
import 'push/models/repos/impl/plugin_push_repository.dart' as _i506;
import 'push/state/push_registration/push_registration_bloc.dart' as _i250;
import 'push/state/push_setting/push_setting_bloc.dart' as _i584;
import 'reminders/state/reminders_settings/reminders_settings_bloc.dart'
    as _i420;
import 'reminders/utils/reminders_coordinator.dart' as _i303;
import 'replace/models/repos/abs/replace_repository.dart' as _i608;
import 'replace/models/repos/impl/rust_replace_repository.dart' as _i1;
import 'replace/state/replace_notes/replace_notes_bloc.dart' as _i1028;
import 'search/state/search/search_bloc.dart' as _i696;
import 'settings/state/about/about_bloc.dart' as _i380;
import 'settings/state/backup_notice/backup_notice_bloc.dart' as _i897;
import 'settings/state/default_notebook/default_notebook_bloc.dart' as _i925;
import 'settings/state/discovery/discovery_settings_bloc.dart' as _i904;
import 'settings/state/icon_themes/icon_themes_bloc.dart' as _i636;
import 'settings/state/licences/licences_bloc.dart' as _i570;
import 'settings/state/sync_settings/sync_settings_bloc.dart' as _i794;
import 'sharing/state/comments/comments_bloc.dart' as _i295;
import 'sharing/state/invitations/invitations_bloc.dart' as _i1;
import 'sharing/state/join_link/join_link_bloc.dart' as _i676;
import 'sharing/state/share_note/share_note_bloc.dart' as _i252;
import 'sharing/state/shared_note_live/shared_note_live_bloc.dart' as _i361;
import 'sharing/state/shared_note_status/shared_note_status_bloc.dart' as _i109;
import 'shell/state/library_pane/library_pane_bloc.dart' as _i612;
import 'shell/state/note_creation/note_creation_bloc.dart' as _i34;
import 'tabs/state/tabs/tabs_bloc.dart' as _i380;
import 'tasks/state/tasks/tasks_bloc.dart' as _i470;
import 'templates/state/daily_template/daily_template_bloc.dart' as _i615;
import 'templates/state/template_picker/template_picker_bloc.dart' as _i828;
import 'themes/state/theme_gallery/theme_gallery_bloc.dart' as _i569;
import 'themes/state/theme_picker/theme_picker_bloc.dart' as _i149;
import 'window/models/repos/abs/note_window_repository.dart' as _i612;
import 'window/models/repos/impl/platform_note_window_repository.dart' as _i271;
import 'window/state/note_window/note_window_bloc.dart' as _i973;

const String _development = 'development';
const String _production = 'production';
const String _integration = 'integration';

extension GetItInjectableX on _i174.GetIt {
  // initializes the registration of main-scope dependencies inside of GetIt
  Future<_i174.GetIt> init({
    String? environment,
    _i526.EnvironmentFilter? environmentFilter,
  }) async {
    final gh = _i526.GetItHelper(this, environment, environmentFilter);
    final coreModule = _$CoreModule();
    final pushModule = _$PushModule();
    final crashModule = _$CrashModule();
    final appModule = _$AppModule();
    final windowModule = _$WindowModule();
    gh.factory<_i612.LibraryPaneBloc>(() => _i612.LibraryPaneBloc());
    gh.singleton<_i609.LayoutClassNotifier>(() => _i609.LayoutClassNotifier());
    gh.lazySingleton<_i183.ImagePicker>(() => coreModule.imagePicker());
    gh.lazySingleton<_i825.ExternalChanges>(() => _i825.ExternalChanges());
    gh.lazySingleton<_i846.LibraryFolderMigration>(
      () => _i846.LibraryFolderMigration(),
    );
    gh.lazySingleton<_i608.WidgetReloader>(() => _i608.WidgetReloader());
    gh.lazySingleton<_i901.KansoPushHostApi>(() => pushModule.hostApi());
    gh.lazySingleton<_i984.DiagnosticsRepository>(
      () => _i1062.RustDiagnosticsRepository(),
    );
    gh.lazySingleton<_i930.NotePlaceRepository>(
      () => _i503.MemoryNotePlaceRepository(),
    );
    gh.lazySingleton<_i874.ReminderNotifier>(
      () => _i740.ChannelReminderNotifier(),
    );
    gh.lazySingleton<_i1056.HapticsRepository>(
      () => _i438.FlutterHapticsRepository(),
    );
    gh.lazySingleton<_i272.WebLinkRepository>(
      () => _i635.UrlLauncherWebLinkRepository(),
    );
    await gh.singletonAsync<_i392.CrashFolder>(
      () => crashModule.crashFolder(),
      registerFor: {_development, _production},
      preResolve: true,
    );
    gh.lazySingleton<_i227.AuthSessionRepository>(
      () => _i680.PluginAuthSessionRepository(),
    );
    gh.lazySingleton<_i202.SystemShareRepository>(
      () => _i254.FlutterSystemShareRepository(),
    );
    gh.lazySingleton<_i80.AboutRepository>(
      () => _i489.PackageInfoAboutRepository(gh<_i909.PatchRepository>()),
    );
    gh.lazySingleton<_i279.DeviceLockRepository>(
      () => _i64.PluginDeviceLockRepository(),
    );
    await gh.singletonAsync<_i72.LibraryFolder>(
      () => coreModule.libraryFolder(gh<_i846.LibraryFolderMigration>()),
      registerFor: {_development, _production},
      preResolve: true,
    );
    gh.lazySingleton<_i909.PatchRepository>(
      () => _i989.ShorebirdPatchRepository(),
    );
    gh.lazySingleton<_i951.LicencesRepository>(
      () => _i1025.FlutterLicencesRepository(),
    );
    gh.singleton<_i289.DesignGalleryAccess>(
      () => appModule.closedGallery,
      registerFor: {_production, _integration},
    );
    gh.lazySingleton<_i26.ImportFilesRepository>(
      () => _i196.ChannelImportFilesRepository(),
    );
    gh.singleton<_i289.DesignGalleryAccess>(
      () => appModule.developmentGallery,
      registerFor: {_development},
    );
    gh.factory<_i570.LicencesBloc>(
      () => _i570.LicencesBloc(gh<_i951.LicencesRepository>()),
    );
    await gh.singletonAsync<_i72.LibraryFolder>(
      () => coreModule.temporaryLibraryFolder(),
      registerFor: {_integration},
      preResolve: true,
    );
    await gh.singletonAsync<_i392.CrashFolder>(
      () => crashModule.temporaryCrashFolder(),
      registerFor: {_integration},
      preResolve: true,
    );
    gh.lazySingleton<_i978.WidgetColorsRepository>(
      () => _i2.FileWidgetColorsRepository(
        gh<_i72.LibraryFolder>(),
        gh<_i608.WidgetReloader>(),
      ),
    );
    gh.lazySingleton<_i384.PreferencesRepository>(
      () => _i207.FilePreferencesRepository(gh<_i72.LibraryFolder>()),
    );
    gh.factory<_i223.WidgetColorsBloc>(
      () => _i223.WidgetColorsBloc(gh<_i978.WidgetColorsRepository>()),
    );
    gh.lazySingleton<_i900.MediaPickerRepository>(
      () => _i598.PlatformMediaPickerRepository(gh<_i183.ImagePicker>()),
    );
    gh.lazySingleton<_i1070.CrashContextRepository>(
      () => _i756.PlatformCrashContextRepository(gh<_i909.PatchRepository>()),
    );
    gh.factory<_i330.PrivacyCoverBloc>(
      () => _i330.PrivacyCoverBloc(gh<_i279.DeviceLockRepository>()),
    );
    gh.factory<_i380.AboutBloc>(
      () => _i380.AboutBloc(gh<_i80.AboutRepository>()),
    );
    gh.factory<_i897.BackupNoticeBloc>(
      () => _i897.BackupNoticeBloc(gh<_i384.PreferencesRepository>()),
    );
    gh.factory<_i925.DefaultNotebookBloc>(
      () => _i925.DefaultNotebookBloc(gh<_i384.PreferencesRepository>()),
    );
    gh.lazySingleton<_i288.CrashReportRepository>(
      () => crashModule.crashReports(
        gh<_i392.CrashFolder>(),
        gh<_i1070.CrashContextRepository>(),
      ),
    );
    gh.singleton<_i388.KansoRouter>(
      () => _i388.KansoRouter(
        gh<_i289.DesignGalleryAccess>(),
        gh<_i609.LayoutClassNotifier>(),
      ),
    );
    await gh.singletonAsync<_i612.NoteWindowRepository>(
      () => windowModule.noteWindows(gh<_i72.LibraryFolder>()),
      preResolve: true,
    );
    await gh.singletonAsync<_i455.KansoLibrary>(
      () => coreModule.library(gh<_i72.LibraryFolder>()),
      preResolve: true,
    );
    gh.lazySingleton<_i38.LibraryRepository>(
      () => _i152.RustLibraryRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i825.ExternalChanges>(),
      ),
    );
    gh.lazySingleton<_i985.UndoNoticeBloc>(
      () => _i985.UndoNoticeBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i1056.HapticsRepository>(),
      ),
    );
    gh.lazySingleton<_i673.DiscoveryRepository>(
      () => _i913.RustDiscoveryRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i815.CrashReportsBloc>(
      () => _i815.CrashReportsBloc(gh<_i288.CrashReportRepository>()),
    );
    gh.lazySingleton<_i983.KansoLocks>(
      () => coreModule.locks(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i591.KansoSync>(
      () => coreModule.sync(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i578.KansoAccount>(
      () => coreModule.account(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i102.KansoAccountSettings>(
      () => coreModule.accountSettings(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i283.KansoApprovals>(
      () => coreModule.approvals(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i832.KansoRenderer>(
      () => coreModule.renderer(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i315.KansoShared>(
      () => coreModule.shared(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i783.KansoGallery>(
      () => coreModule.gallery(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i732.RendererRepository>(
      () => _i820.RustRendererRepository(gh<_i832.KansoRenderer>()),
    );
    gh.lazySingleton<_i704.PaletteRepository>(
      () => _i97.RustPaletteRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i696.SearchBloc>(
      () => _i696.SearchBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i673.DiscoveryRepository>(),
      ),
    );
    gh.lazySingleton<_i937.PushRepository>(
      () => _i506.PluginPushRepository(
        gh<_i901.KansoPushHostApi>(),
        gh<_i591.KansoSync>(),
      ),
    );
    gh.factory<_i636.IconThemesBloc>(
      () => _i636.IconThemesBloc(gh<_i732.RendererRepository>()),
    );
    gh.lazySingleton<_i465.ApprovalsRepository>(
      () => _i99.RustApprovalsRepository(gh<_i283.KansoApprovals>()),
    );
    gh.lazySingleton<_i242.ImportRepository>(
      () => _i903.RustImportRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i380.TabsBloc>(
      () => _i380.TabsBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
    );
    gh.lazySingleton<_i23.CaptureRepository>(
      () => _i824.RustCaptureRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i72.LibraryFolder>(),
        gh<_i825.ExternalChanges>(),
      ),
    );
    gh.lazySingleton<_i169.JournalRepository>(
      () => _i212.RustJournalRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i974.HistoryRepository>(
      () => _i1065.RustHistoryRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i904.DiscoverySettingsBloc>(
      () => _i904.DiscoverySettingsBloc(
        gh<_i673.DiscoveryRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
    );
    gh.lazySingleton<_i235.BoardRepository>(
      () => _i418.RustBoardRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i403.NoteHistoryBloc>(
      () => _i403.NoteHistoryBloc(
        gh<_i974.HistoryRepository>(),
        gh<_i38.LibraryRepository>(),
        gh<_i202.SystemShareRepository>(),
      ),
    );
    gh.lazySingleton<_i460.TemplatesRepository>(
      () => _i213.RustTemplatesRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i250.PushRegistrationBloc>(
      () => _i250.PushRegistrationBloc(gh<_i937.PushRepository>()),
    );
    gh.factory<_i584.PushSettingBloc>(
      () => _i584.PushSettingBloc(gh<_i937.PushRepository>()),
    );
    gh.factory<_i618.NoteBloc>(
      () => _i618.NoteBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i930.NotePlaceRepository>(),
      ),
    );
    gh.factory<_i553.BoardBloc>(
      () => _i553.BoardBloc(
        gh<_i235.BoardRepository>(),
        gh<_i38.LibraryRepository>(),
      ),
    );
    gh.lazySingleton<_i979.EditorRepository>(
      () => _i606.RustEditorRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i861.LibraryIndexBloc>(
      () => _i861.LibraryIndexBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i907.ListNavigationBloc>(
      () => _i907.ListNavigationBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i991.NewNotebookBloc>(
      () => _i991.NewNotebookBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i34.NoteCreationBloc>(
      () => _i34.NoteCreationBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i930.NotePlaceRepository>(),
        gh<_i384.PreferencesRepository>(),
        gh<_i23.CaptureRepository>(),
        gh<_i900.MediaPickerRepository>(),
      ),
    );
    gh.lazySingleton<_i58.RemindersRepository>(
      () => _i429.RustRemindersRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i513.ImportFlowBloc>(
      () => _i513.ImportFlowBloc(
        gh<_i242.ImportRepository>(),
        gh<_i26.ImportFilesRepository>(),
      ),
    );
    gh.factory<_i979.OpenedDocumentsBloc>(
      () => _i979.OpenedDocumentsBloc(
        gh<_i242.ImportRepository>(),
        gh<_i26.ImportFilesRepository>(),
      ),
    );
    gh.lazySingleton<_i608.ReplaceRepository>(
      () => _i1.RustReplaceRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i351.NoteListBloc>(
      () => _i351.NoteListBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i384.PreferencesRepository>(),
        gh<_i1056.HapticsRepository>(),
        gh<_i985.UndoNoticeBloc>(),
      ),
    );
    gh.lazySingleton<_i1002.BackupsRepository>(
      () => _i270.RustBackupsRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i686.JournalBloc>(
      () => _i686.JournalBloc(
        gh<_i169.JournalRepository>(),
        gh<_i460.TemplatesRepository>(),
      ),
    );
    gh.factory<_i365.GraphBloc>(
      () => _i365.GraphBloc(gh<_i673.DiscoveryRepository>()),
    );
    gh.factory<_i779.NoteDiscoveryBloc>(
      () => _i779.NoteDiscoveryBloc(gh<_i673.DiscoveryRepository>()),
    );
    gh.factory<_i1060.DailyNoteBloc>(
      () => _i1060.DailyNoteBloc(gh<_i23.CaptureRepository>()),
    );
    gh.factory<_i551.CaptureInboxBloc>(
      () => _i551.CaptureInboxBloc(gh<_i23.CaptureRepository>()),
    );
    gh.factory<_i964.EditorBloc>(
      () => _i964.EditorBloc(
        gh<_i979.EditorRepository>(),
        gh<_i272.WebLinkRepository>(),
        gh<_i900.MediaPickerRepository>(),
      ),
    );
    gh.factoryParam<_i272.NavigationHistoryBloc, _i380.TabsBloc?, dynamic>(
      (_tabs, _) =>
          _i272.NavigationHistoryBloc(gh<_i38.LibraryRepository>(), _tabs),
    );
    gh.lazySingleton<_i1035.AccountRepository>(
      () =>
          _i450.RustAccountRepository(gh<_i578.KansoAccount>())
            ..startRelayingPasskeys(),
    );
    gh.factory<_i470.TasksBloc>(
      () => _i470.TasksBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i1056.HapticsRepository>(),
        gh<_i985.UndoNoticeBloc>(),
      ),
    );
    gh.factory<_i390.CalendarBloc>(
      () => _i390.CalendarBloc(gh<_i169.JournalRepository>()),
    );
    gh.lazySingleton<_i314.AccountSettingsRepository>(
      () =>
          _i350.RustAccountSettingsRepository(gh<_i102.KansoAccountSettings>()),
    );
    gh.lazySingleton<_i601.SyncRepository>(
      () => _i564.RustSyncRepository(gh<_i591.KansoSync>()),
    );
    gh.lazySingleton<_i333.SessionRepository>(
      () => _i899.RustSessionRepository(gh<_i983.KansoLocks>()),
    );
    gh.factory<_i615.DailyTemplateBloc>(
      () => _i615.DailyTemplateBloc(gh<_i460.TemplatesRepository>()),
    );
    gh.factory<_i828.TemplatePickerBloc>(
      () => _i828.TemplatePickerBloc(gh<_i460.TemplatesRepository>()),
    );
    gh.factory<_i797.UnlockBloc>(
      () => _i797.UnlockBloc(
        gh<_i333.SessionRepository>(),
        gh<_i279.DeviceLockRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
    );
    gh.lazySingleton<_i550.GalleryRepository>(
      () => _i692.RustGalleryRepository(gh<_i783.KansoGallery>()),
    );
    gh.factory<_i204.PaletteBloc>(
      () => _i204.PaletteBloc(gh<_i704.PaletteRepository>()),
    );
    gh.factory<_i770.BackupsBloc>(
      () => _i770.BackupsBloc(gh<_i1002.BackupsRepository>()),
    );
    gh.lazySingleton<_i374.SharedNotesRepository>(
      () => _i628.RustSharedNotesRepository(gh<_i315.KansoShared>()),
    );
    gh.factory<_i552.SignInBloc>(
      () => _i552.SignInBloc(
        gh<_i1035.AccountRepository>(),
        gh<_i601.SyncRepository>(),
        gh<_i227.AuthSessionRepository>(),
      ),
    );
    gh.factory<_i274.DeviceApprovalBloc>(
      () => _i274.DeviceApprovalBloc(gh<_i465.ApprovalsRepository>()),
    );
    gh.factory<_i435.PreviewBloc>(
      () => _i435.PreviewBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i272.WebLinkRepository>(),
        gh<_i985.UndoNoticeBloc>(),
        gh<_i1056.HapticsRepository>(),
      ),
    );
    gh.lazySingleton<_i303.RemindersCoordinator>(
      () => _i303.RemindersCoordinator(
        gh<_i384.PreferencesRepository>(),
        gh<_i58.RemindersRepository>(),
        gh<_i874.ReminderNotifier>(),
        gh<_i38.LibraryRepository>(),
      ),
    );
    gh.factory<_i185.SyncStatusBloc>(
      () => _i185.SyncStatusBloc(gh<_i601.SyncRepository>()),
    );
    gh.factory<_i794.SyncSettingsBloc>(
      () => _i794.SyncSettingsBloc(gh<_i601.SyncRepository>()),
    );
    gh.factory<_i1028.ReplaceNotesBloc>(
      () => _i1028.ReplaceNotesBloc(gh<_i608.ReplaceRepository>()),
    );
    gh.factory<_i39.ChangePasswordBloc>(
      () => _i39.ChangePasswordBloc(gh<_i333.SessionRepository>()),
    );
    gh.factory<_i215.LockRemoveBloc>(
      () => _i215.LockRemoveBloc(gh<_i333.SessionRepository>()),
    );
    gh.factory<_i249.LockSetupBloc>(
      () => _i249.LockSetupBloc(gh<_i333.SessionRepository>()),
    );
    gh.factory<_i569.ThemeGalleryBloc>(
      () => _i569.ThemeGalleryBloc(gh<_i550.GalleryRepository>()),
    );
    gh.factory<_i1059.RecoverBloc>(
      () => _i1059.RecoverBloc(
        gh<_i1035.AccountRepository>(),
        gh<_i601.SyncRepository>(),
      ),
    );
    gh.factory<_i973.NoteWindowBloc>(
      () => _i973.NoteWindowBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i333.SessionRepository>(),
      ),
    );
    gh.factory<_i149.ThemePickerBloc>(
      () => _i149.ThemePickerBloc(
        gh<_i732.RendererRepository>(),
        gh<_i550.GalleryRepository>(),
      ),
    );
    gh.factory<_i35.RecoveryKeyBloc>(
      () => _i35.RecoveryKeyBloc(gh<_i1035.AccountRepository>()),
    );
    gh.factory<_i858.UnlockBloc>(
      () => _i858.UnlockBloc(gh<_i1035.AccountRepository>()),
    );
    gh.factory<_i168.AccountBloc>(
      () => _i168.AccountBloc(gh<_i1035.AccountRepository>()),
    );
    gh.factory<_i252.ShareNoteBloc>(
      () => _i252.ShareNoteBloc(
        gh<_i374.SharedNotesRepository>(),
        gh<_i202.SystemShareRepository>(),
      ),
    );
    gh.factory<_i295.CommentsBloc>(
      () => _i295.CommentsBloc(gh<_i374.SharedNotesRepository>()),
    );
    gh.factory<_i1.InvitationsBloc>(
      () => _i1.InvitationsBloc(gh<_i374.SharedNotesRepository>()),
    );
    gh.factory<_i676.JoinLinkBloc>(
      () => _i676.JoinLinkBloc(gh<_i374.SharedNotesRepository>()),
    );
    gh.factory<_i361.SharedNoteLiveBloc>(
      () => _i361.SharedNoteLiveBloc(gh<_i374.SharedNotesRepository>()),
    );
    gh.factory<_i109.SharedNoteStatusBloc>(
      () => _i109.SharedNoteStatusBloc(gh<_i374.SharedNotesRepository>()),
    );
    gh.factory<_i551.AppearanceBloc>(
      () => _i551.AppearanceBloc(
        gh<_i384.PreferencesRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i550.GalleryRepository>(),
      ),
    );
    gh.factory<_i645.AccountDetailsBloc>(
      () => _i645.AccountDetailsBloc(
        gh<_i314.AccountSettingsRepository>(),
        gh<_i1035.AccountRepository>(),
        gh<_i227.AuthSessionRepository>(),
      ),
    );
    gh.factory<_i477.BiometricsBloc>(
      () => _i477.BiometricsBloc(
        gh<_i333.SessionRepository>(),
        gh<_i279.DeviceLockRepository>(),
      ),
    );
    gh.factory<_i57.LockStatusBloc>(
      () => _i57.LockStatusBloc(
        gh<_i333.SessionRepository>(),
        gh<_i279.DeviceLockRepository>(),
      ),
    );
    gh.factory<_i420.RemindersSettingsBloc>(
      () => _i420.RemindersSettingsBloc(
        gh<_i384.PreferencesRepository>(),
        gh<_i874.ReminderNotifier>(),
        gh<_i303.RemindersCoordinator>(),
      ),
    );
    return this;
  }
}

class _$CoreModule extends _i418.CoreModule {}

class _$PushModule extends _i78.PushModule {}

class _$CrashModule extends _i1016.CrashModule {}

class _$AppModule extends _i216.AppModule {}

class _$WindowModule extends _i271.WindowModule {}
