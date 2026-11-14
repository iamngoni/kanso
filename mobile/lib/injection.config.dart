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
import 'package:kanso_dictation/kanso_dictation.dart' as _i33;
import 'package:kanso_push/kanso_push.dart' as _i901;

import 'account/state/account_details/account_details_bloc.dart' as _i645;
import 'account/state/recover/recover_bloc.dart' as _i1059;
import 'account/state/recovery_key/recovery_key_bloc.dart' as _i35;
import 'account/state/sign_in/sign_in_bloc.dart' as _i552;
import 'account/state/unlock/unlock_bloc.dart' as _i858;
import 'agent_tasks/state/agent_review/agent_review_bloc.dart' as _i54;
import 'agent_tasks/state/agent_session/agent_session_bloc.dart' as _i243;
import 'agent_tasks/state/agent_settings/agent_settings_cubit.dart' as _i733;
import 'agent_tasks/state/agent_tasks/agent_tasks_bloc.dart' as _i939;
import 'ai_settings/state/ai_settings/ai_settings_bloc.dart' as _i182;
import 'ai_settings/state/ai_status/ai_status_bloc.dart' as _i1030;
import 'ask/state/ask/ask_bloc.dart' as _i216;
import 'assist/state/assist/assist_bloc.dart' as _i567;
import 'backups/models/repos/abs/backups_repository.dart' as _i1002;
import 'backups/models/repos/impl/rust_backups_repository.dart' as _i270;
import 'backups/state/backups/backups_bloc.dart' as _i770;
import 'board/state/board/board_bloc.dart' as _i553;
import 'canvas/models/repos/abs/canvas_repository.dart' as _i727;
import 'canvas/models/repos/impl/rust_canvas_repository.dart' as _i890;
import 'canvas/state/canvas/canvas_bloc.dart' as _i601;
import 'capture/models/repos/abs/media_picker_repository.dart' as _i900;
import 'capture/models/repos/impl/platform_media_picker_repository.dart'
    as _i598;
import 'capture/state/capture_task/capture_task_cubit.dart' as _i161;
import 'capture/state/daily_note/daily_note_bloc.dart' as _i1060;
import 'core/bridge/api/account.dart' as _i578;
import 'core/bridge/api/account_settings.dart' as _i102;
import 'core/bridge/api/ai.dart' as _i35;
import 'core/bridge/api/approvals.dart' as _i283;
import 'core/bridge/api/feedback.dart' as _i442;
import 'core/bridge/api/gallery.dart' as _i783;
import 'core/bridge/api/library.dart' as _i455;
import 'core/bridge/api/locks.dart' as _i983;
import 'core/bridge/api/metrics.dart' as _i575;
import 'core/bridge/api/renderer.dart' as _i832;
import 'core/bridge/api/shared.dart' as _i315;
import 'core/bridge/api/sync.dart' as _i591;
import 'core/bridge/api/themes.dart' as _i737;
import 'core/configs/app_module.dart' as _i216;
import 'core/configs/core_module.dart' as _i418;
import 'core/models/data/design_gallery_access.dart' as _i289;
import 'core/models/data/library_folder.dart' as _i72;
import 'core/models/repos/abs/about_repository.dart' as _i80;
import 'core/models/repos/abs/account_repository.dart' as _i1035;
import 'core/models/repos/abs/account_settings_repository.dart' as _i314;
import 'core/models/repos/abs/agent_task_notifier.dart' as _i687;
import 'core/models/repos/abs/agent_tasks_repository.dart' as _i214;
import 'core/models/repos/abs/ai_repository.dart' as _i84;
import 'core/models/repos/abs/approvals_repository.dart' as _i465;
import 'core/models/repos/abs/auth_session_repository.dart' as _i227;
import 'core/models/repos/abs/board_repository.dart' as _i235;
import 'core/models/repos/abs/capture_repository.dart' as _i23;
import 'core/models/repos/abs/diagnostics_repository.dart' as _i984;
import 'core/models/repos/abs/digest_repository.dart' as _i622;
import 'core/models/repos/abs/file_picker_repository.dart' as _i205;
import 'core/models/repos/abs/gallery_repository.dart' as _i550;
import 'core/models/repos/abs/google_font_repository.dart' as _i1009;
import 'core/models/repos/abs/haptics_repository.dart' as _i1056;
import 'core/models/repos/abs/journal_repository.dart' as _i169;
import 'core/models/repos/abs/library_repository.dart' as _i38;
import 'core/models/repos/abs/licences_repository.dart' as _i951;
import 'core/models/repos/abs/notices_repository.dart' as _i691;
import 'core/models/repos/abs/patch_repository.dart' as _i909;
import 'core/models/repos/abs/preferences_repository.dart' as _i384;
import 'core/models/repos/abs/reminder_notifier.dart' as _i874;
import 'core/models/repos/abs/reminders_repository.dart' as _i58;
import 'core/models/repos/abs/renderer_repository.dart' as _i732;
import 'core/models/repos/abs/review_prompt_repository.dart' as _i765;
import 'core/models/repos/abs/schedule_repository.dart' as _i507;
import 'core/models/repos/abs/session_repository.dart' as _i333;
import 'core/models/repos/abs/shared_notebooks_repository.dart' as _i564;
import 'core/models/repos/abs/shared_notes_repository.dart' as _i374;
import 'core/models/repos/abs/store_review_repository.dart' as _i690;
import 'core/models/repos/abs/sync_repository.dart' as _i601;
import 'core/models/repos/abs/system_share_repository.dart' as _i202;
import 'core/models/repos/abs/templates_repository.dart' as _i460;
import 'core/models/repos/abs/themes_repository.dart' as _i40;
import 'core/models/repos/abs/whats_new_repository.dart' as _i435;
import 'core/models/repos/abs/widget_colors_repository.dart' as _i978;
import 'core/models/repos/impl/channel_reminder_notifier.dart' as _i740;
import 'core/models/repos/impl/file_google_font_repository.dart' as _i8;
import 'core/models/repos/impl/file_preferences_repository.dart' as _i207;
import 'core/models/repos/impl/file_widget_colors_repository.dart' as _i2;
import 'core/models/repos/impl/flutter_haptics_repository.dart' as _i438;
import 'core/models/repos/impl/flutter_licences_repository.dart' as _i1025;
import 'core/models/repos/impl/flutter_system_share_repository.dart' as _i254;
import 'core/models/repos/impl/google_font_source.dart' as _i434;
import 'core/models/repos/impl/in_app_store_review_repository.dart' as _i155;
import 'core/models/repos/impl/package_info_about_repository.dart' as _i489;
import 'core/models/repos/impl/plugin_auth_session_repository.dart' as _i680;
import 'core/models/repos/impl/preferences_review_prompt_repository.dart'
    as _i842;
import 'core/models/repos/impl/rust_account_repository.dart' as _i450;
import 'core/models/repos/impl/rust_account_settings_repository.dart' as _i350;
import 'core/models/repos/impl/rust_agent_tasks_repository.dart' as _i770;
import 'core/models/repos/impl/rust_ai_repository.dart' as _i500;
import 'core/models/repos/impl/rust_approvals_repository.dart' as _i99;
import 'core/models/repos/impl/rust_board_repository.dart' as _i418;
import 'core/models/repos/impl/rust_capture_repository.dart' as _i824;
import 'core/models/repos/impl/rust_diagnostics_repository.dart' as _i1062;
import 'core/models/repos/impl/rust_digest_repository.dart' as _i472;
import 'core/models/repos/impl/rust_gallery_repository.dart' as _i692;
import 'core/models/repos/impl/rust_journal_repository.dart' as _i212;
import 'core/models/repos/impl/rust_library_repository.dart' as _i152;
import 'core/models/repos/impl/rust_notices_repository.dart' as _i136;
import 'core/models/repos/impl/rust_reminders_repository.dart' as _i429;
import 'core/models/repos/impl/rust_renderer_repository.dart' as _i820;
import 'core/models/repos/impl/rust_schedule_repository.dart' as _i294;
import 'core/models/repos/impl/rust_session_repository.dart' as _i899;
import 'core/models/repos/impl/rust_shared_notebooks_repository.dart' as _i899;
import 'core/models/repos/impl/rust_shared_notes_repository.dart' as _i628;
import 'core/models/repos/impl/rust_sync_repository.dart' as _i564;
import 'core/models/repos/impl/rust_templates_repository.dart' as _i213;
import 'core/models/repos/impl/rust_whats_new_repository.dart' as _i420;
import 'core/models/repos/impl/shorebird_patch_repository.dart' as _i989;
import 'core/models/repos/impl/system_agent_task_notifier.dart' as _i238;
import 'core/models/repos/impl/system_file_picker_repository.dart' as _i448;
import 'core/models/repos/impl/themes_repository_impl.dart' as _i48;
import 'core/router/kanso_router.dart' as _i388;
import 'core/router/layout_class_notifier.dart' as _i609;
import 'core/services/external_changes.dart' as _i825;
import 'core/services/library_folder_migration.dart' as _i846;
import 'core/services/preview_theme_fitter.dart' as _i550;
import 'core/services/review_prompt.dart' as _i849;
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
import 'dictation/models/repos/abs/dictation_repository.dart' as _i872;
import 'dictation/models/repos/abs/dictation_text_rules.dart' as _i426;
import 'dictation/models/repos/impl/dictation_module.dart' as _i429;
import 'dictation/models/repos/impl/platform_dictation_repository.dart'
    as _i1029;
import 'dictation/models/repos/impl/rust_dictation_text_rules.dart' as _i38;
import 'dictation/state/dictation/dictation_bloc.dart' as _i657;
import 'dictation/state/voice_note/voice_note_bloc.dart' as _i284;
import 'discovery/models/repos/abs/discovery_repository.dart' as _i673;
import 'discovery/models/repos/impl/rust_discovery_repository.dart' as _i913;
import 'editor/models/repos/abs/editor_repository.dart' as _i979;
import 'editor/models/repos/abs/web_link_repository.dart' as _i272;
import 'editor/models/repos/abs/when_repository.dart' as _i741;
import 'editor/models/repos/impl/rust_editor_repository.dart' as _i606;
import 'editor/models/repos/impl/rust_when_repository.dart' as _i773;
import 'editor/models/repos/impl/url_launcher_web_link_repository.dart'
    as _i635;
import 'editor/state/editor/editor_bloc.dart' as _i964;
import 'embargo/models/repos/abs/embargo_repository.dart' as _i454;
import 'embargo/models/repos/impl/rust_embargo_repository.dart' as _i284;
import 'feedback/models/repos/abs/feedback_repository.dart' as _i15;
import 'feedback/models/repos/abs/feedback_screenshot_repository.dart' as _i146;
import 'feedback/models/repos/impl/photo_feedback_screenshot_repository.dart'
    as _i596;
import 'feedback/models/repos/impl/rust_feedback_repository.dart' as _i894;
import 'feedback/state/my_feedback/my_feedback_bloc.dart' as _i519;
import 'feedback/state/send_feedback/send_feedback_bloc.dart' as _i455;
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
import 'lock/state/files_migration/files_migration_bloc.dart' as _i115;
import 'lock/state/interrupted_lock/interrupted_lock_bloc.dart' as _i582;
import 'lock/state/lock_remove/lock_remove_bloc.dart' as _i215;
import 'lock/state/lock_setup/lock_setup_bloc.dart' as _i249;
import 'lock/state/lock_status/lock_status_bloc.dart' as _i57;
import 'lock/state/privacy_cover/privacy_cover_bloc.dart' as _i330;
import 'lock/state/unlock/unlock_bloc.dart' as _i797;
import 'metrics/models/repos/abs/usage_metrics_repository.dart' as _i314;
import 'metrics/models/repos/impl/rust_usage_metrics_repository.dart' as _i931;
import 'note/models/repos/abs/note_place_repository.dart' as _i930;
import 'note/models/repos/impl/memory_note_place_repository.dart' as _i503;
import 'note/state/note/note_bloc.dart' as _i618;
import 'note/state/note_assist_intent/note_assist_intent_bloc.dart' as _i247;
import 'note/state/note_discovery/note_discovery_bloc.dart' as _i779;
import 'note/state/note_mode_intent/note_mode_intent_bloc.dart' as _i908;
import 'note/state/preview/preview_bloc.dart' as _i435;
import 'note/state/resurface/resurface_bloc.dart' as _i170;
import 'note/state/word_goal_intent/word_goal_intent_bloc.dart' as _i304;
import 'note_links/models/repos/abs/note_links_repository.dart' as _i393;
import 'note_links/models/repos/impl/rust_note_links_repository.dart' as _i199;
import 'note_list/state/note_list/note_list_bloc.dart' as _i351;
import 'notebook_home/models/repos/abs/notebook_home_repository.dart' as _i257;
import 'notebook_home/models/repos/impl/rust_notebook_home_repository.dart'
    as _i826;
import 'notebook_home/state/notebook_home/notebook_home_bloc.dart' as _i343;
import 'notices/state/notices_cubit.dart' as _i919;
import 'palette/models/repos/abs/palette_repository.dart' as _i704;
import 'palette/models/repos/impl/rust_palette_repository.dart' as _i97;
import 'palette/state/palette/palette_bloc.dart' as _i204;
import 'pdf_reader/models/repos/abs/pdf_reader_repository.dart' as _i367;
import 'pdf_reader/models/repos/impl/rust_pdf_reader_repository.dart' as _i758;
import 'pdf_reader/state/pdf_reader/pdf_reader_bloc.dart' as _i818;
import 'people/models/repos/abs/people_repository.dart' as _i755;
import 'people/models/repos/impl/rust_people_repository.dart' as _i1053;
import 'publish/models/repos/abs/publish_repository.dart' as _i1047;
import 'publish/models/repos/impl/rust_publish_repository.dart' as _i99;
import 'publish/state/publish/publish_bloc.dart' as _i614;
import 'push/configs/push_module.dart' as _i78;
import 'push/models/repos/abs/push_repository.dart' as _i937;
import 'push/models/repos/impl/plugin_push_repository.dart' as _i506;
import 'push/state/push_registration/push_registration_bloc.dart' as _i250;
import 'push/state/push_setting/push_setting_bloc.dart' as _i584;
import 'recording/models/repos/abs/player_repository.dart' as _i217;
import 'recording/models/repos/abs/recording_repository.dart' as _i28;
import 'recording/models/repos/impl/rust_player_repository.dart' as _i441;
import 'recording/models/repos/impl/rust_recording_repository.dart' as _i804;
import 'recording/services/recording_audio_session.dart' as _i133;
import 'recording/state/player/player_bloc.dart' as _i186;
import 'recording/state/recording/recording_bloc.dart' as _i181;
import 'recording_speakers/models/repos/abs/recording_speakers_repository.dart'
    as _i150;
import 'recording_speakers/models/repos/impl/rust_recording_speakers_repository.dart'
    as _i50;
import 'recording_summary/models/repos/abs/recording_summary_repository.dart'
    as _i34;
import 'recording_summary/models/repos/impl/rust_recording_summary_repository.dart'
    as _i821;
import 'reminders/state/reminders_settings/reminders_settings_bloc.dart'
    as _i420;
import 'reminders/utils/reminders_coordinator.dart' as _i303;
import 'replace/models/repos/abs/replace_repository.dart' as _i608;
import 'replace/models/repos/impl/rust_replace_repository.dart' as _i1;
import 'replace/state/replace_notes/replace_notes_bloc.dart' as _i1028;
import 'schedule/state/schedule/schedule_bloc.dart' as _i518;
import 'search/state/search/search_bloc.dart' as _i696;
import 'settings/state/about/about_bloc.dart' as _i380;
import 'settings/state/backup_notice/backup_notice_bloc.dart' as _i897;
import 'settings/state/default_notebook/default_notebook_bloc.dart' as _i925;
import 'settings/state/discovery/discovery_settings_bloc.dart' as _i904;
import 'settings/state/icon_themes/icon_themes_bloc.dart' as _i636;
import 'settings/state/licences/licences_bloc.dart' as _i570;
import 'settings/state/sync_settings/sync_settings_bloc.dart' as _i794;
import 'settings/state/word_goal_settings/word_goal_settings_bloc.dart'
    as _i893;
import 'sharing/state/comments/comments_bloc.dart' as _i295;
import 'sharing/state/invitations/invitations_bloc.dart' as _i1;
import 'sharing/state/join_link/join_link_bloc.dart' as _i676;
import 'sharing/state/notebook_sharing/notebook_sharing_bloc.dart' as _i869;
import 'sharing/state/share_note/share_note_bloc.dart' as _i252;
import 'sharing/state/share_notebook/share_notebook_bloc.dart' as _i375;
import 'sharing/state/shared_note_live/shared_note_live_bloc.dart' as _i361;
import 'sharing/state/shared_note_status/shared_note_status_bloc.dart' as _i109;
import 'sharing/state/shared_notebooks/shared_notebooks_bloc.dart' as _i137;
import 'shell/state/focus_mode/focus_mode_bloc.dart' as _i427;
import 'shell/state/library_pane/library_pane_bloc.dart' as _i612;
import 'shell/state/note_creation/note_creation_bloc.dart' as _i34;
import 'tabs/state/tabs/tabs_bloc.dart' as _i380;
import 'tasks/state/tasks/tasks_bloc.dart' as _i470;
import 'templates/state/daily_template/daily_template_bloc.dart' as _i615;
import 'templates/state/template_picker/template_picker_bloc.dart' as _i828;
import 'themes/state/icon_themes/icon_themes_bloc.dart' as _i778;
import 'themes/state/theme_gallery/theme_gallery_bloc.dart' as _i569;
import 'themes/state/theme_picker/theme_picker_bloc.dart' as _i149;
import 'triage/models/repos/abs/triage_repository.dart' as _i731;
import 'triage/models/repos/impl/rust_triage_repository.dart' as _i975;
import 'triage/state/triage/triage_bloc.dart' as _i1007;
import 'voice_capture/models/repos/abs/voice_capture_repository.dart' as _i525;
import 'voice_capture/models/repos/impl/rust_voice_capture_repository.dart'
    as _i718;
import 'voice_capture/state/voice_drops/voice_drops_bloc.dart' as _i712;
import 'voice_settings/models/repos/abs/voice_settings_repository.dart'
    as _i1027;
import 'voice_settings/models/repos/impl/rust_voice_settings_repository.dart'
    as _i759;
import 'voice_settings/state/voice_settings/voice_settings_bloc.dart' as _i64;
import 'whats_new/state/whats_new_cubit.dart' as _i594;
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
    final dictationModule = _$DictationModule();
    final pushModule = _$PushModule();
    final crashModule = _$CrashModule();
    final appModule = _$AppModule();
    final windowModule = _$WindowModule();
    gh.factory<_i247.NoteAssistIntentBloc>(() => _i247.NoteAssistIntentBloc());
    gh.factory<_i908.NoteModeIntentBloc>(() => _i908.NoteModeIntentBloc());
    gh.factory<_i304.WordGoalIntentBloc>(() => _i304.WordGoalIntentBloc());
    gh.factory<_i612.LibraryPaneBloc>(() => _i612.LibraryPaneBloc());
    gh.singleton<_i609.LayoutClassNotifier>(() => _i609.LayoutClassNotifier());
    gh.lazySingleton<_i183.ImagePicker>(() => coreModule.imagePicker());
    gh.lazySingleton<_i825.ExternalChanges>(() => _i825.ExternalChanges());
    gh.lazySingleton<_i846.LibraryFolderMigration>(
      () => _i846.LibraryFolderMigration(),
    );
    gh.lazySingleton<_i608.WidgetReloader>(() => _i608.WidgetReloader());
    gh.lazySingleton<_i33.KansoDictation>(() => dictationModule.kansoDictation);
    gh.lazySingleton<_i901.KansoPushHostApi>(() => pushModule.hostApi());
    gh.lazySingleton<_i133.RecordingAudioSession>(
      () => _i133.RecordingAudioSession(),
    );
    gh.lazySingleton<_i984.DiagnosticsRepository>(
      () => _i1062.RustDiagnosticsRepository(),
    );
    gh.lazySingleton<_i930.NotePlaceRepository>(
      () => _i503.MemoryNotePlaceRepository(),
    );
    gh.lazySingleton<_i8.GoogleFontRegistrar>(
      () => _i8.FlutterGoogleFontRegistrar(),
    );
    gh.lazySingleton<_i874.ReminderNotifier>(
      () => _i740.ChannelReminderNotifier(),
    );
    gh.lazySingleton<_i1056.HapticsRepository>(
      () => _i438.FlutterHapticsRepository(),
    );
    gh.lazySingleton<_i205.FilePickerRepository>(
      () => _i448.SystemFilePickerRepository(),
    );
    gh.lazySingleton<_i272.WebLinkRepository>(
      () => _i635.UrlLauncherWebLinkRepository(),
    );
    gh.lazySingleton<_i434.GoogleFontStylesheetParser>(
      () => _i434.GoogleFontCssParser(),
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
    gh.lazySingleton<_i690.StoreReviewRepository>(
      () => _i155.InAppStoreReviewRepository(),
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
    gh.lazySingleton<_i687.AgentTaskNotifier>(
      () => _i238.SystemAgentTaskNotifier(gh<_i874.ReminderNotifier>()),
    );
    gh.lazySingleton<_i741.WhenRepository>(() => _i773.RustWhenRepository());
    gh.lazySingleton<_i146.FeedbackScreenshotRepository>(
      () => _i596.PhotoFeedbackScreenshotRepository(gh<_i183.ImagePicker>()),
    );
    gh.singleton<_i289.DesignGalleryAccess>(
      () => appModule.developmentGallery,
      registerFor: {_development},
    );
    gh.lazySingleton<_i872.DictationRepository>(
      () => _i1029.PlatformDictationRepository(gh<_i33.KansoDictation>()),
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
    gh.lazySingleton<_i434.GoogleFontSource>(
      () => _i434.GoogleFontHttpSource(gh<_i434.GoogleFontStylesheetParser>()),
    );
    gh.lazySingleton<_i1070.CrashContextRepository>(
      () => _i756.PlatformCrashContextRepository(gh<_i909.PatchRepository>()),
    );
    gh.factory<_i330.PrivacyCoverBloc>(
      () => _i330.PrivacyCoverBloc(gh<_i279.DeviceLockRepository>()),
    );
    gh.lazySingleton<_i765.ReviewPromptRepository>(
      () => _i842.PreferencesReviewPromptRepository(
        gh<_i384.PreferencesRepository>(),
      ),
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
    gh.factory<_i427.FocusModeBloc>(
      () => _i427.FocusModeBloc(gh<_i384.PreferencesRepository>()),
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
    gh.lazySingleton<_i1009.GoogleFontRepository>(
      () => _i8.FileGoogleFontRepository(
        gh<_i72.LibraryFolder>(),
        gh<_i434.GoogleFontSource>(),
        gh<_i8.GoogleFontRegistrar>(),
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
    gh.lazySingleton<_i257.NotebookHomeRepository>(
      () => _i826.RustNotebookHomeRepository(gh<_i455.KansoLibrary>()),
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
    gh.lazySingleton<_i727.CanvasRepository>(
      () => _i890.RustCanvasRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i673.DiscoveryRepository>(
      () => _i913.RustDiscoveryRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i815.CrashReportsBloc>(
      () => _i815.CrashReportsBloc(gh<_i288.CrashReportRepository>()),
    );
    gh.factory<_i601.CanvasBloc>(
      () => _i601.CanvasBloc(gh<_i727.CanvasRepository>()),
    );
    gh.lazySingleton<_i731.TriageRepository>(
      () => _i975.RustTriageRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i983.KansoLocks>(
      () => coreModule.locks(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i35.KansoAi>(
      () => coreModule.ai(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i575.KansoMetrics>(
      () => coreModule.metrics(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i442.KansoFeedback>(
      () => coreModule.feedback(gh<_i455.KansoLibrary>()),
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
    gh.lazySingleton<_i737.KansoThemes>(
      () => coreModule.themes(gh<_i455.KansoLibrary>()),
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
    gh.lazySingleton<_i564.SharedNotebooksRepository>(
      () => _i899.RustSharedNotebooksRepository(gh<_i315.KansoShared>()),
    );
    gh.lazySingleton<_i849.ReviewPrompt>(
      () => _i849.ReviewPrompt(
        gh<_i765.ReviewPromptRepository>(),
        gh<_i690.StoreReviewRepository>(),
        gh<_i80.AboutRepository>(),
        gh<_i288.CrashReportRepository>(),
      ),
    );
    gh.lazySingleton<_i454.EmbargoRepository>(
      () => _i284.RustEmbargoRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i704.PaletteRepository>(
      () => _i97.RustPaletteRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i507.ScheduleRepository>(
      () => _i294.RustScheduleRepository(gh<_i455.KansoLibrary>()),
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
    gh.lazySingleton<_i691.NoticesRepository>(
      () => _i136.RustNoticesRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i636.IconThemesBloc>(
      () => _i636.IconThemesBloc(gh<_i732.RendererRepository>()),
    );
    gh.lazySingleton<_i465.ApprovalsRepository>(
      () => _i99.RustApprovalsRepository(gh<_i283.KansoApprovals>()),
    );
    gh.lazySingleton<_i1047.PublishRepository>(
      () => _i99.RustPublishRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i242.ImportRepository>(
      () => _i903.RustImportRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i435.WhatsNewRepository>(
      () => _i420.RustWhatsNewRepository(gh<_i455.KansoLibrary>()),
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
    gh.factory<_i161.CaptureTaskCubit>(
      () => _i161.CaptureTaskCubit(
        gh<_i23.CaptureRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
    );
    gh.factory<_i170.ResurfaceBloc>(
      () => _i170.ResurfaceBloc(
        gh<_i673.DiscoveryRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
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
    gh.lazySingleton<_i84.AiRepository>(
      () => _i500.RustAiRepository(
        gh<_i35.KansoAi>(),
        gh<_i609.LayoutClassNotifier>(),
      ),
    );
    gh.lazySingleton<_i622.DigestRepository>(
      () => _i472.RustDigestRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i35.KansoAi>(),
      ),
    );
    gh.lazySingleton<_i34.RecordingSummaryRepository>(
      () => _i821.RustRecordingSummaryRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i553.BoardBloc>(
      () => _i553.BoardBloc(
        gh<_i235.BoardRepository>(),
        gh<_i38.LibraryRepository>(),
      ),
    );
    gh.factory<_i1007.TriageBloc>(
      () => _i1007.TriageBloc(
        gh<_i731.TriageRepository>(),
        gh<_i985.UndoNoticeBloc>(),
        gh<_i1056.HapticsRepository>(),
      ),
    );
    gh.lazySingleton<_i367.PdfReaderRepository>(
      () => _i758.RustPdfReaderRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i182.AiSettingsBloc>(
      () => _i182.AiSettingsBloc(gh<_i84.AiRepository>()),
    );
    gh.factory<_i1030.AiStatusBloc>(
      () => _i1030.AiStatusBloc(gh<_i84.AiRepository>()),
    );
    gh.factory<_i216.AskBloc>(() => _i216.AskBloc(gh<_i84.AiRepository>()));
    gh.factory<_i567.AssistBloc>(
      () => _i567.AssistBloc(gh<_i84.AiRepository>()),
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
    gh.factory<_i893.WordGoalSettingsBloc>(
      () => _i893.WordGoalSettingsBloc(gh<_i38.LibraryRepository>()),
    );
    gh.lazySingleton<_i393.NoteLinksRepository>(
      () => _i199.RustNoteLinksRepository(gh<_i455.KansoLibrary>()),
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
    gh.lazySingleton<_i525.VoiceCaptureRepository>(
      () => _i718.RustVoiceCaptureRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i72.LibraryFolder>(),
        gh<_i825.ExternalChanges>(),
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
    gh.lazySingleton<_i150.RecordingSpeakersRepository>(
      () => _i50.RustRecordingSpeakersRepository(gh<_i455.KansoLibrary>()),
    );
    gh.factory<_i686.JournalBloc>(
      () => _i686.JournalBloc(
        gh<_i169.JournalRepository>(),
        gh<_i460.TemplatesRepository>(),
      ),
    );
    gh.lazySingleton<_i217.PlayerRepository>(
      () => _i441.RustPlayerRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i133.RecordingAudioSession>(),
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
    gh.lazySingleton<_i15.FeedbackRepository>(
      () => _i894.RustFeedbackRepository(
        gh<_i442.KansoFeedback>(),
        gh<_i1070.CrashContextRepository>(),
        gh<_i288.CrashReportRepository>(),
      ),
    );
    gh.factoryParam<_i272.NavigationHistoryBloc, _i380.TabsBloc?, dynamic>(
      (_tabs, _) =>
          _i272.NavigationHistoryBloc(gh<_i38.LibraryRepository>(), _tabs),
    );
    gh.lazySingleton<_i28.RecordingRepository>(
      () => _i804.RustRecordingRepository(
        gh<_i455.KansoLibrary>(),
        gh<_i72.LibraryFolder>(),
        gh<_i825.ExternalChanges>(),
        gh<_i133.RecordingAudioSession>(),
      ),
    );
    gh.lazySingleton<_i1035.AccountRepository>(
      () =>
          _i450.RustAccountRepository(gh<_i578.KansoAccount>())
            ..startRelayingPasskeys(),
    );
    gh.lazySingleton<_i181.RecordingBloc>(
      () => _i181.RecordingBloc(gh<_i28.RecordingRepository>()),
    );
    gh.factory<_i343.NotebookHomeBloc>(
      () => _i343.NotebookHomeBloc(gh<_i257.NotebookHomeRepository>()),
    );
    gh.factory<_i186.PlayerBloc>(
      () => _i186.PlayerBloc(gh<_i217.PlayerRepository>()),
    );
    gh.factory<_i919.NoticesCubit>(
      () => _i919.NoticesCubit(gh<_i691.NoticesRepository>()),
    );
    gh.factory<_i470.TasksBloc>(
      () => _i470.TasksBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i1056.HapticsRepository>(),
        gh<_i985.UndoNoticeBloc>(),
      ),
    );
    gh.lazySingleton<_i755.PeopleRepository>(
      () => _i1053.RustPeopleRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i1027.VoiceSettingsRepository>(
      () => _i759.RustVoiceSettingsRepository(gh<_i455.KansoLibrary>()),
    );
    gh.lazySingleton<_i303.RemindersCoordinator>(
      () => _i303.RemindersCoordinator(
        gh<_i384.PreferencesRepository>(),
        gh<_i58.RemindersRepository>(),
        gh<_i874.ReminderNotifier>(),
        gh<_i38.LibraryRepository>(),
        gh<_i622.DigestRepository>(),
      ),
    );
    gh.factory<_i390.CalendarBloc>(
      () => _i390.CalendarBloc(gh<_i169.JournalRepository>()),
    );
    gh.lazySingleton<_i314.AccountSettingsRepository>(
      () =>
          _i350.RustAccountSettingsRepository(gh<_i102.KansoAccountSettings>()),
    );
    gh.lazySingleton<_i214.AgentTasksRepository>(
      () => _i770.RustAgentTasksRepository(gh<_i35.KansoAi>()),
    );
    gh.lazySingleton<_i601.SyncRepository>(
      () => _i564.RustSyncRepository(gh<_i591.KansoSync>()),
    );
    gh.factory<_i518.ScheduleBloc>(
      () => _i518.ScheduleBloc(
        gh<_i507.ScheduleRepository>(),
        gh<_i38.LibraryRepository>(),
        gh<_i1056.HapticsRepository>(),
        gh<_i985.UndoNoticeBloc>(),
      ),
    );
    gh.factory<_i284.VoiceNoteBloc>(
      () => _i284.VoiceNoteBloc(gh<_i525.VoiceCaptureRepository>()),
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
    gh.factory<_i420.RemindersSettingsBloc>(
      () => _i420.RemindersSettingsBloc(
        gh<_i384.PreferencesRepository>(),
        gh<_i874.ReminderNotifier>(),
        gh<_i303.RemindersCoordinator>(),
      ),
    );
    gh.lazySingleton<_i314.UsageMetricsRepository>(
      () => _i931.RustUsageMetricsRepository(gh<_i575.KansoMetrics>()),
    );
    gh.factory<_i797.UnlockBloc>(
      () => _i797.UnlockBloc(
        gh<_i333.SessionRepository>(),
        gh<_i279.DeviceLockRepository>(),
        gh<_i384.PreferencesRepository>(),
      ),
    );
    gh.factory<_i594.WhatsNewCubit>(
      () => _i594.WhatsNewCubit(gh<_i435.WhatsNewRepository>()),
    );
    gh.lazySingleton<_i40.ThemesRepository>(
      () => _i48.ThemesRepositoryImpl(gh<_i737.KansoThemes>()),
    );
    gh.lazySingleton<_i550.GalleryRepository>(
      () => _i692.RustGalleryRepository(gh<_i783.KansoGallery>()),
    );
    gh.factory<_i618.NoteBloc>(
      () => _i618.NoteBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i930.NotePlaceRepository>(),
        gh<_i333.SessionRepository>(),
      ),
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
    gh.factory<_i375.ShareNotebookBloc>(
      () => _i375.ShareNotebookBloc(
        gh<_i564.SharedNotebooksRepository>(),
        gh<_i374.SharedNotesRepository>(),
      ),
    );
    gh.factory<_i185.SyncStatusBloc>(
      () => _i185.SyncStatusBloc(gh<_i601.SyncRepository>()),
    );
    gh.factory<_i794.SyncSettingsBloc>(
      () => _i794.SyncSettingsBloc(gh<_i601.SyncRepository>()),
    );
    gh.factory<_i614.PublishBloc>(
      () => _i614.PublishBloc(gh<_i1047.PublishRepository>()),
    );
    gh.factory<_i435.PreviewBloc>(
      () => _i435.PreviewBloc(
        gh<_i38.LibraryRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i40.ThemesRepository>(),
        gh<_i272.WebLinkRepository>(),
        gh<_i985.UndoNoticeBloc>(),
        gh<_i1056.HapticsRepository>(),
      ),
    );
    gh.factory<_i455.SendFeedbackBloc>(
      () => _i455.SendFeedbackBloc(
        gh<_i15.FeedbackRepository>(),
        gh<_i146.FeedbackScreenshotRepository>(),
      ),
    );
    gh.factory<_i1028.ReplaceNotesBloc>(
      () => _i1028.ReplaceNotesBloc(gh<_i608.ReplaceRepository>()),
    );
    gh.factory<_i712.VoiceDropsBloc>(
      () => _i712.VoiceDropsBloc(
        gh<_i525.VoiceCaptureRepository>(),
        gh<_i872.DictationRepository>(),
        gh<_i1027.VoiceSettingsRepository>(),
      ),
    );
    gh.factory<_i519.MyFeedbackBloc>(
      () => _i519.MyFeedbackBloc(gh<_i15.FeedbackRepository>()),
    );
    gh.factory<_i39.ChangePasswordBloc>(
      () => _i39.ChangePasswordBloc(gh<_i333.SessionRepository>()),
    );
    gh.factory<_i115.FilesMigrationBloc>(
      () => _i115.FilesMigrationBloc(gh<_i333.SessionRepository>()),
    );
    gh.factory<_i582.InterruptedLockBloc>(
      () => _i582.InterruptedLockBloc(gh<_i333.SessionRepository>()),
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
    gh.factory<_i550.PreviewThemeFitter>(
      () => _i550.PreviewThemeFitter(gh<_i40.ThemesRepository>()),
    );
    gh.factory<_i64.VoiceSettingsBloc>(
      () => _i64.VoiceSettingsBloc(
        gh<_i1027.VoiceSettingsRepository>(),
        gh<_i872.DictationRepository>(),
      ),
    );
    gh.factory<_i551.AppearanceBloc>(
      () => _i551.AppearanceBloc(
        gh<_i384.PreferencesRepository>(),
        gh<_i732.RendererRepository>(),
        gh<_i550.GalleryRepository>(),
        gh<_i40.ThemesRepository>(),
        googleFonts: gh<_i1009.GoogleFontRepository>(),
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
    gh.factory<_i869.NotebookSharingBloc>(
      () => _i869.NotebookSharingBloc(
        gh<_i564.SharedNotebooksRepository>(),
        gh<_i374.SharedNotesRepository>(),
      ),
    );
    gh.factory<_i137.SharedNotebooksBloc>(
      () => _i137.SharedNotebooksBloc(
        gh<_i564.SharedNotebooksRepository>(),
        gh<_i374.SharedNotesRepository>(),
      ),
    );
    gh.factory<_i252.ShareNoteBloc>(
      () => _i252.ShareNoteBloc(
        gh<_i374.SharedNotesRepository>(),
        gh<_i202.SystemShareRepository>(),
      ),
    );
    gh.factory<_i939.AgentTasksBloc>(
      () => _i939.AgentTasksBloc(
        gh<_i214.AgentTasksRepository>(),
        gh<_i687.AgentTaskNotifier>(),
      ),
    );
    gh.factory<_i818.PdfReaderBloc>(
      () => _i818.PdfReaderBloc(
        gh<_i367.PdfReaderRepository>(),
        gh<_i384.PreferencesRepository>(),
        gh<_i333.SessionRepository>(),
      ),
    );
    gh.lazySingleton<_i426.DictationTextRules>(
      () => _i38.RustDictationTextRules(gh<_i1027.VoiceSettingsRepository>()),
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
    gh.factory<_i54.AgentReviewBloc>(
      () => _i54.AgentReviewBloc(gh<_i214.AgentTasksRepository>()),
    );
    gh.factory<_i243.AgentSessionBloc>(
      () => _i243.AgentSessionBloc(gh<_i214.AgentTasksRepository>()),
    );
    gh.factory<_i733.AgentSettingsCubit>(
      () => _i733.AgentSettingsCubit(gh<_i214.AgentTasksRepository>()),
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
    gh.factory<_i657.DictationBloc>(
      () => _i657.DictationBloc(
        gh<_i872.DictationRepository>(),
        gh<_i426.DictationTextRules>(),
      ),
    );
    return this;
  }
}

class _$CoreModule extends _i418.CoreModule {}

class _$DictationModule extends _i429.DictationModule {}

class _$PushModule extends _i78.PushModule {}

class _$CrashModule extends _i1016.CrashModule {}

class _$AppModule extends _i216.AppModule {}

class _$WindowModule extends _i271.WindowModule {}
