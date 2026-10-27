// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

// **************************************************************************
// InjectableConfigGenerator
// **************************************************************************

// ignore_for_file: type=lint
// coverage:ignore-file

// ignore_for_file: no_leading_underscores_for_library_prefixes

import 'package:get_it/get_it.dart' as _i174;
import 'package:injectable/injectable.dart' as _i526;

import 'core/bridge/api/library.dart' as _i455;
import 'core/bridge/api/locks.dart' as _i983;
import 'core/bridge/api/renderer.dart' as _i832;
import 'core/bridge/api/sync.dart' as _i591;
import 'core/configs/app_module.dart' as _i216;
import 'core/configs/core_module.dart' as _i418;
import 'core/models/data/design_gallery_access.dart' as _i289;
import 'core/models/data/library_folder.dart' as _i72;
import 'core/models/repos/abs/about_repository.dart' as _i80;
import 'core/models/repos/abs/diagnostics_repository.dart' as _i984;
import 'core/models/repos/abs/file_picker_repository.dart' as _i205;
import 'core/models/repos/abs/haptics_repository.dart' as _i1056;
import 'core/models/repos/abs/library_repository.dart' as _i38;
import 'core/models/repos/abs/licences_repository.dart' as _i951;
import 'core/models/repos/abs/patch_repository.dart' as _i909;
import 'core/models/repos/abs/preferences_repository.dart' as _i384;
import 'core/models/repos/abs/renderer_repository.dart' as _i732;
import 'core/models/repos/abs/session_repository.dart' as _i333;
import 'core/models/repos/abs/sync_repository.dart' as _i601;
import 'core/models/repos/abs/themes_repository.dart' as _i40;
import 'core/models/repos/impl/file_preferences_repository.dart' as _i207;
import 'core/models/repos/impl/flutter_haptics_repository.dart' as _i438;
import 'core/models/repos/impl/flutter_licences_repository.dart' as _i1025;
import 'core/models/repos/impl/package_info_about_repository.dart' as _i489;
import 'core/models/repos/impl/rust_diagnostics_repository.dart' as _i1062;
import 'core/models/repos/impl/rust_library_repository.dart' as _i152;
import 'core/models/repos/impl/rust_renderer_repository.dart' as _i820;
import 'core/models/repos/impl/rust_session_repository.dart' as _i899;
import 'core/models/repos/impl/rust_sync_repository.dart' as _i564;
import 'core/models/repos/impl/shorebird_patch_repository.dart' as _i989;
import 'core/models/repos/impl/system_file_picker_repository.dart' as _i448;
import 'core/models/repos/impl/themes_repository_impl.dart' as _i48;
import 'core/router/kanso_router.dart' as _i388;
import 'core/router/layout_class_notifier.dart' as _i609;
import 'core/state/appearance/appearance_bloc.dart' as _i551;
import 'core/state/library_index/library_index_bloc.dart' as _i861;
import 'core/state/list_navigation/list_navigation_bloc.dart' as _i907;
import 'core/state/navigation_history/navigation_history_bloc.dart' as _i272;
import 'core/state/sync_status/sync_status_bloc.dart' as _i185;
import 'crash/configs/crash_module.dart' as _i1016;
import 'crash/models/data/crash_folder.dart' as _i392;
import 'crash/models/repos/abs/crash_context_repository.dart' as _i1070;
import 'crash/models/repos/abs/crash_report_repository.dart' as _i288;
import 'crash/models/repos/impl/platform_crash_context_repository.dart'
    as _i756;
import 'crash/state/crash_reports/crash_reports_bloc.dart' as _i815;
import 'note/state/note/note_bloc.dart' as _i618;
import 'note_list/state/note_list/note_list_bloc.dart' as _i351;
import 'search/state/search/search_bloc.dart' as _i696;
import 'settings/state/about/about_bloc.dart' as _i380;
import 'settings/state/backup_notice/backup_notice_bloc.dart' as _i897;
import 'settings/state/licences/licences_bloc.dart' as _i570;
import 'shell/state/library_pane/library_pane_bloc.dart' as _i612;
import 'themes/state/icon_themes/icon_themes_bloc.dart' as _i778;
import 'themes/state/theme_picker/theme_picker_bloc.dart' as _i149;

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
    final crashModule = _$CrashModule();
    final appModule = _$AppModule();
    gh.factory<_i612.LibraryPaneBloc>(() => _i612.LibraryPaneBloc());
    gh.singleton<_i609.LayoutClassNotifier>(() => _i609.LayoutClassNotifier());
    gh.lazySingleton<_i984.DiagnosticsRepository>(
      () => _i1062.RustDiagnosticsRepository(),
    );
    gh.lazySingleton<_i1056.HapticsRepository>(
      () => _i438.FlutterHapticsRepository(),
    );
    gh.lazySingleton<_i205.FilePickerRepository>(
      () => _i448.SystemFilePickerRepository(),
    );
    gh.lazySingleton<_i40.ThemesRepository>(() => _i48.ThemesRepositoryImpl());
    await gh.singletonAsync<_i72.LibraryFolder>(
      () => coreModule.libraryFolder(),
      registerFor: {_development, _production},
      preResolve: true,
    );
    await gh.singletonAsync<_i392.CrashFolder>(
      () => crashModule.crashFolder(),
      registerFor: {_development, _production},
      preResolve: true,
    );
    gh.lazySingleton<_i80.AboutRepository>(
      () => _i489.PackageInfoAboutRepository(gh<_i909.PatchRepository>()),
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
    gh.factory<_i778.IconThemesBloc>(
      () => _i778.IconThemesBloc(
        gh<_i40.ThemesRepository>(),
        gh<_i205.FilePickerRepository>(),
      ),
    );
    gh.factory<_i149.ThemePickerBloc>(
      () => _i149.ThemePickerBloc(
        gh<_i40.ThemesRepository>(),
        gh<_i205.FilePickerRepository>(),
      ),
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
    gh.lazySingleton<_i384.PreferencesRepository>(
      () => _i207.FilePreferencesRepository(gh<_i72.LibraryFolder>()),
    );
    gh.factory<_i551.AppearanceBloc>(
      () => _i551.AppearanceBloc(
        gh<_i384.PreferencesRepository>(),
        gh<_i40.ThemesRepository>(),
      ),
    );
    gh.lazySingleton<_i1070.CrashContextRepository>(
      () => _i756.PlatformCrashContextRepository(gh<_i909.PatchRepository>()),
    );
    gh.factory<_i380.AboutBloc>(
      () => _i380.AboutBloc(gh<_i80.AboutRepository>()),
    );
    gh.factory<_i897.BackupNoticeBloc>(
      () => _i897.BackupNoticeBloc(gh<_i384.PreferencesRepository>()),
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
    await gh.singletonAsync<_i455.KansoLibrary>(
      () => coreModule.library(gh<_i72.LibraryFolder>()),
      preResolve: true,
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
    gh.lazySingleton<_i832.KansoRenderer>(
      () => coreModule.renderer(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i732.RendererRepository>(
      () => _i820.RustRendererRepository(gh<_i832.KansoRenderer>()),
    );
    gh.lazySingleton<_i38.LibraryRepository>(
      () => _i152.RustLibraryRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i861.LibraryIndexBloc>(
      () => _i861.LibraryIndexBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i907.ListNavigationBloc>(
      () => _i907.ListNavigationBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i272.NavigationHistoryBloc>(
      () => _i272.NavigationHistoryBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i696.SearchBloc>(
      () => _i696.SearchBloc(gh<_i38.LibraryRepository>()),
    );
    gh.factory<_i351.NoteListBloc>(
      () => _i351.NoteListBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i384.PreferencesRepository>(),
        gh<_i1056.HapticsRepository>(),
      ),
    );
    gh.factory<_i618.NoteBloc>(
      () => _i618.NoteBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i732.RendererRepository>(),
      ),
    );
    gh.lazySingleton<_i601.SyncRepository>(
      () => _i564.RustSyncRepository(gh<_i591.KansoSync>()),
    );
    gh.lazySingleton<_i333.SessionRepository>(
      () => _i899.RustSessionRepository(gh<_i983.KansoLocks>()),
    );
    gh.factory<_i185.SyncStatusBloc>(
      () => _i185.SyncStatusBloc(gh<_i601.SyncRepository>()),
    );
    return this;
  }
}

class _$CoreModule extends _i418.CoreModule {}

class _$CrashModule extends _i1016.CrashModule {}

class _$AppModule extends _i216.AppModule {}
