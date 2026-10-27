// Kanso — register_page_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/backups/models/repos/abs/backups_repository.dart';
import 'package:kanso/backups/state/backups/backups_bloc.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/history/models/repos/abs/history_repository.dart';
import 'package:kanso/history/state/note_history/note_history_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/library/state/new_notebook/new_notebook_bloc.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/replace/models/repos/abs/replace_repository.dart';
import 'package:kanso/replace/state/replace_notes/replace_notes_bloc.dart';
import 'package:kanso/search/state/search/search_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/icon_themes/icon_themes_bloc.dart';
import 'package:kanso/settings/state/licences/licences_bloc.dart';
import 'package:kanso/sharing/state/comments/comments_bloc.dart';
import 'package:kanso/sharing/state/join_link/join_link_bloc.dart';
import 'package:kanso/sharing/state/share_note/share_note_bloc.dart';
import 'package:kanso/sharing/state/shared_note_live/shared_note_live_bloc.dart';
import 'package:kanso/sharing/state/shared_note_status/shared_note_status_bloc.dart';
import 'package:kanso/tasks/state/tasks/tasks_bloc.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_about_repository.dart';
import 'fixture_gallery_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_licences_repository.dart';
import 'fixture_parity_repositories.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_shared_notes_repository.dart';
import 'fixture_system_share_repository.dart';
import 'seeded_appearance_bloc.dart';

/// Registers, for a test that builds the app's pages, the BLoCs the pages
/// make for themselves, over [library]; they are taken out again when the
/// test ends. The lists share [undo], or a notice of their own.
///
/// [gallery], [shared] and [system] are the theme gallery, the shared notes
/// and the clipboard those BLoCs use; fixtures of them are made when a test
/// gives none.
void registerPageBlocs(
  FixtureLibraryRepository library, [
  FixtureRendererRepository? renderer,
  UndoNoticeBloc? undo,
  FixtureGalleryRepository? gallery,
  FixtureSharedNotesRepository? shared,
  FixtureSystemShareRepository? system,
  FixtureHistoryRepository? history,
  FixtureReplaceRepository? replace,
  FixtureBackupsRepository? backups,
]) {
  final themes = gallery ?? FixtureGalleryRepository();
  final sharing = shared ?? FixtureSharedNotesRepository();
  final clipboard = system ?? FixtureSystemShareRepository();
  final noteHistory = history ?? FixtureHistoryRepository();
  final noteReplace = replace ?? FixtureReplaceRepository();
  final libraryBackups = backups ?? FixtureBackupsRepository();
  final haptics = RecordingHaptics();
  final undoNotice = undo ?? UndoNoticeBloc(library, haptics);
  if (undo == null) {
    addTearDown(() => unawaited(undoNotice.close()));
  }
  if (getIt.isRegistered<NoteListBloc>()) {
    getIt.unregister<NoteListBloc>();
  }
  getIt.registerFactory<NoteListBloc>(
    () => NoteListBloc(
      library,
      ForgetfulPreferencesRepository(),
      haptics,
      undoNotice,
    ),
  );
  if (getIt.isRegistered<TasksBloc>()) {
    getIt.unregister<TasksBloc>();
  }
  getIt.registerFactory<TasksBloc>(
    () => TasksBloc(library, haptics, undoNotice),
  );
  if (getIt.isRegistered<NoteBloc>()) {
    getIt.unregister<NoteBloc>();
  }
  getIt.registerFactory<NoteBloc>(
    () => NoteBloc(
      library,
      renderer ?? FixtureRendererRepository(),
      MemoryNotePlaceRepository(),
    ),
  );
  if (getIt.isRegistered<NewNotebookBloc>()) {
    getIt.unregister<NewNotebookBloc>();
  }
  getIt.registerFactory<NewNotebookBloc>(() => NewNotebookBloc(library));
  if (getIt.isRegistered<SearchBloc>()) {
    getIt.unregister<SearchBloc>();
  }
  getIt.registerFactory<SearchBloc>(() => SearchBloc(library));
  if (getIt.isRegistered<AboutBloc>()) {
    getIt.unregister<AboutBloc>();
  }
  getIt.registerFactory<AboutBloc>(() => AboutBloc(FixtureAboutRepository()));
  if (getIt.isRegistered<LicencesBloc>()) {
    getIt.unregister<LicencesBloc>();
  }
  getIt.registerFactory<LicencesBloc>(
    () => LicencesBloc(FixtureLicencesRepository()),
  );
  if (getIt.isRegistered<ThemePickerBloc>()) {
    getIt.unregister<ThemePickerBloc>();
  }
  final themeRepository = themes ?? FixtureThemesRepository();
  final filePicker = files ?? FixtureFilePickerRepository();
  getIt.registerFactory<ThemePickerBloc>(
    () => ThemePickerBloc(renderer ?? FixtureRendererRepository(), themes),
  );
  if (getIt.isRegistered<ThemeGalleryBloc>()) {
    getIt.unregister<ThemeGalleryBloc>();
  }
  getIt.registerFactory<ThemeGalleryBloc>(() => ThemeGalleryBloc(themes));
  if (getIt.isRegistered<ShareNoteBloc>()) {
    getIt.unregister<ShareNoteBloc>();
  }
  getIt.registerFactory<ShareNoteBloc>(
    () => ShareNoteBloc(sharing, clipboard),
  );
  if (getIt.isRegistered<JoinLinkBloc>()) {
    getIt.unregister<JoinLinkBloc>();
  }
  getIt.registerFactory<JoinLinkBloc>(() => JoinLinkBloc(sharing));
  if (getIt.isRegistered<SharedNoteLiveBloc>()) {
    getIt.unregister<SharedNoteLiveBloc>();
  }
  getIt.registerFactory<SharedNoteLiveBloc>(() => SharedNoteLiveBloc(sharing));
  if (getIt.isRegistered<CommentsBloc>()) {
    getIt.unregister<CommentsBloc>();
  }
  getIt.registerFactory<CommentsBloc>(() => CommentsBloc(sharing));
  if (getIt.isRegistered<SharedNoteStatusBloc>()) {
    getIt.unregister<SharedNoteStatusBloc>();
  }
  getIt.registerFactory<SharedNoteStatusBloc>(
    () => SharedNoteStatusBloc(sharing),
  );
  if (getIt.isRegistered<IconThemesBloc>()) {
    getIt.unregister<IconThemesBloc>();
  }
  getIt.registerFactory<IconThemesBloc>(
    () => IconThemesBloc(renderer ?? FixtureRendererRepository()),
  );
  if (getIt.isRegistered<NoteHistoryBloc>()) {
    getIt.unregister<NoteHistoryBloc>();
  }
  getIt.registerFactory<NoteHistoryBloc>(
    () => NoteHistoryBloc(noteHistory, library, clipboard),
  );
  if (getIt.isRegistered<HistoryRepository>()) {
    getIt.unregister<HistoryRepository>();
  }
  getIt.registerLazySingleton<HistoryRepository>(() => noteHistory);
  if (getIt.isRegistered<ReplaceNotesBloc>()) {
    getIt.unregister<ReplaceNotesBloc>();
  }
  getIt.registerFactory<ReplaceNotesBloc>(
    () => ReplaceNotesBloc(noteReplace),
  );
  if (getIt.isRegistered<ReplaceRepository>()) {
    getIt.unregister<ReplaceRepository>();
  }
  getIt.registerLazySingleton<ReplaceRepository>(() => noteReplace);
  if (getIt.isRegistered<BackupsBloc>()) {
    getIt.unregister<BackupsBloc>();
  }
  getIt.registerFactory<BackupsBloc>(
    () => BackupsBloc(libraryBackups),
  );
  if (getIt.isRegistered<BackupsRepository>()) {
    getIt.unregister<BackupsRepository>();
  }
  getIt.registerLazySingleton<BackupsRepository>(() => libraryBackups);
  addTearDown(() => unawaited(getIt.reset()));
}
