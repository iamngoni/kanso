// Kanso — kanso_router.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:go_router/go_router.dart';
import 'package:injectable/injectable.dart';

import '../../account/views/pages/account_page.dart';
import '../../account/views/pages/recover_page.dart';
import '../../account/views/pages/recovery_key_page.dart';
import '../../account/views/pages/sign_in_page.dart';
import '../../account/views/pages/unlock_page.dart';
import '../../backups/views/pages/backups_page.dart';
import '../../capture/views/pages/daily_note_page.dart';
import '../../capture/views/pages/new_note_page.dart';
import '../../crash/views/pages/crash_reports_page.dart';
import '../../design/views/pages/design_gallery_page.dart';
import '../../history/views/pages/history_page.dart';
import '../../import/views/pages/import_page.dart';
import '../../note/views/pages/note_page.dart';
import '../../note/views/widgets/no_note_selected_pane.dart';
import '../../note_list/views/pages/note_list_page.dart';
import '../../replace/views/pages/replace_page.dart';
import '../../search/views/pages/search_page.dart';
import '../../settings/views/pages/default_notebook_page.dart';
import '../../settings/views/pages/fonts_page.dart';
import '../../settings/views/pages/icon_themes_page.dart' as icon_picker;
import '../../settings/views/pages/licences_page.dart';
import '../../settings/views/pages/settings_page.dart';
import '../../sharing/views/pages/join_link_page.dart';
import '../../sharing/views/pages/shared_with_you_page.dart';
import '../../shell/views/widgets/browse_shell.dart';
import '../../tasks/views/pages/tasks_page.dart';
import '../../themes/views/pages/icon_themes_page.dart' as icon_themes;
import '../../themes/views/pages/theme_picker_page.dart';
import '../bridge/api/types.dart';
import '../models/data/browse_location.dart';
import '../models/data/design_gallery_access.dart';
import '../models/data/layout_class.dart';
import '../models/data/search_scope_kind.dart';
import 'deep_links.dart';
import 'kanso_routes.dart';
import 'kanso_sheet_page.dart';
import 'layout_class_notifier.dart';

/// The app's routes.
///
/// * Every place and search is under one shell, [BrowseShell], and a note
///   opened from one is that address plus `/note/:noteId`.
/// * Settings is a shell of its own on top of the root navigator, so it
///   opens over what the person was doing and closes back to it, with
///   Theme, Fonts and Licences sliding inside it. It is a sheet on a tablet
///   and a page on a phone.
/// * `/daily` finds or makes today's note and goes to it in All Notes, and
///   `/new` makes a note as the + button does (`?voice=1` to dictate into).
///   Theme, Icons, Icon themes, Fonts, Licences, Crash reports and the
///   account's screens (Account, Sign in, Open your notes, Reset your
///   password, Your recovery key) sliding inside it. It is a sheet on a
///   tablet and a page on a phone.
///   Shared with you, and Join with a link inside it, and Import notes are
///   in that shell too.
/// * `kanso://open/…` links and the short forms (`/note/:id`,
///   `/notebook/:id`) land on the addresses above; an address that is
///   nothing goes to All Notes.
@singleton
class KansoRouter {
  KansoRouter(this._gallery, this.layoutClass) {
    config = _build();
  }

  final DesignGalleryAccess _gallery;

  /// How many panes there is room for; the pages are built again when it
  /// changes.
  final LayoutClassNotifier layoutClass;

  final GlobalKey<NavigatorState> _rootKey = GlobalKey<NavigatorState>(
    debugLabel: 'root',
  );
  final GlobalKey<NavigatorState> _browseKey = GlobalKey<NavigatorState>(
    debugLabel: 'browse',
  );
  final GlobalKey<NavigatorState> _settingsKey = GlobalKey<NavigatorState>(
    debugLabel: 'settings',
  );

  /// The router the app runs with.
  late final GoRouter config;

  GoRouter _build() {
    return GoRouter(
      navigatorKey: _rootKey,
      initialLocation: KansoRoutes.notes,
      refreshListenable: layoutClass,
      // The app's own links carry a scheme and a host, which the router
      // ignores; they are turned into addresses of the app here.
      redirect: (context, state) =>
          state.uri.hasScheme ? redirectLocation(state.uri) : null,
      onException: (context, state, router) => router.go(KansoRoutes.notes),
      routes: <RouteBase>[
        GoRoute(
          path: KansoRoutes.home,
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        GoRoute(
          path: '${KansoRoutes.noteById}/:noteId',
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        GoRoute(
          path: '${KansoRoutes.notebookById}/:notebookId',
          redirect: (context, state) => redirectLocation(state.uri),
        ),
        GoRoute(
          path: '${KansoRoutes.history}/:noteId',
          pageBuilder: (context, state) => CupertinoPage<void>(
            key: state.pageKey,
            child: HistoryPage(noteId: state.pathParameters['noteId']!),
          ),
        ),
        GoRoute(
          path: KansoRoutes.replace,
          pageBuilder: (context, state) => CupertinoPage<void>(
            key: state.pageKey,
            child: const ReplacePage(),
          ),
        ),
        ShellRoute(
          navigatorKey: _browseKey,
          builder: (context, state, child) => BrowseShell(
            layout: layoutClass,
            location: BrowseLocation.parse(state.uri),
            designGallery: _gallery.available,
            child: child,
          ),
          routes: <RouteBase>[
            // Inside the shell, so going on to the note keeps the browsing
            // screens (and their navigator) as they are.
            GoRoute(
              path: KansoRoutes.daily,
              pageBuilder: (context, state) => NoTransitionPage<void>(
                key: state.pageKey,
                child: const DailyNotePage(),
              ),
            ),
            GoRoute(
              path: KansoRoutes.newNote,
              pageBuilder: (context, state) => NoTransitionPage<void>(
                key: state.pageKey,
                child: NewNotePage(
                  voice:
                      state.uri.queryParameters[KansoRoutes.voiceParameter] ==
                      '1',
                ),
              ),
            ),
            _placeRoute(KansoRoutes.notes, (_) => const ListPlace.all()),
            _placeRoute(KansoRoutes.pinned, (_) => const ListPlace.pinned()),
            _placeRoute(KansoRoutes.recent, (_) => const ListPlace.recent()),
            _placeRoute(KansoRoutes.trash, (_) => const ListPlace.trash()),
            _placeRoute(
              '${KansoRoutes.notebooks}/:notebookId',
              (state) =>
                  ListPlace.notebook(id: state.pathParameters['notebookId']!),
            ),
            _placeRoute(
              '${KansoRoutes.tags}/:tagId',
              (state) => ListPlace.tag(id: state.pathParameters['tagId']!),
            ),
            GoRoute(
              path: KansoRoutes.tasks,
              pageBuilder: (context, state) =>
                  _listPage(state, single: const TasksPage()),
              routes: <RouteBase>[_noteRoute()],
            ),
            GoRoute(
              path: KansoRoutes.search,
              pageBuilder: (context, state) => _listPage(
                state,
                single: SearchPage(
                  query: state.uri.queryParameters['q'] ?? '',
                  scope: SearchScopeKind.values.firstWhere(
                    (kind) => kind.name == state.uri.queryParameters['scope'],
                    orElse: () => SearchScopeKind.all,
                  ),
                  notebookId: state.uri.queryParameters['in'],
                ),
              ),
              routes: <RouteBase>[_noteRoute()],
            ),
          ],
        ),
        ShellRoute(
          navigatorKey: _settingsKey,
          pageBuilder: (context, state, child) => layoutClass.tabletDevice
              ? KansoSheetPage<void>(key: state.pageKey, child: child)
              : CupertinoPage<void>(key: state.pageKey, child: child),
          routes: <RouteBase>[
            GoRoute(
              path: KansoRoutes.settings,
              pageBuilder: (context, state) => CupertinoPage<void>(
                key: state.pageKey,
                child: const SettingsPage(),
              ),
              routes: <RouteBase>[
                GoRoute(
                  path: 'notebook',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const DefaultNotebookPage(),
                  ),
                ),
                GoRoute(
                  path: 'theme',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const ThemePickerPage(),
                  ),
                ),
                GoRoute(
                  path: 'icons',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const icon_picker.IconThemesPage(),
                  ),
                ),
                GoRoute(
                  path: 'icon-themes',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const icon_themes.IconThemesPage(),
                  ),
                ),
                GoRoute(
                  path: 'fonts',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: FontsPage(
                      atTextSize:
                          state.uri.queryParameters[KansoRoutes
                              .fontsAtParameter] ==
                          KansoRoutes.fontsAtSize,
                    ),
                  ),
                ),
                GoRoute(
                  path: 'licences',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const LicencesPage(),
                  ),
                ),
                GoRoute(
                  path: 'crash-reports',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const CrashReportsPage(),
                  ),
                ),
                GoRoute(
                  path: 'backups',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const BackupsPage(),
                  ),
                ),
                GoRoute(
                  path: 'account',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const AccountPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/sign-in',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const SignInPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/unlock',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const UnlockPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/recover',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: RecoverPage(
                      server: state
                          .uri
                          .queryParameters[KansoRoutes.serverParameter],
                    ),
                  ),
                ),
                GoRoute(
                  path: 'account/recovery-key',
                  pageBuilder: (context, state) {
                    final extra = state.extra;
                    return CupertinoPage<void>(
                      key: state.pageKey,
                      child: RecoveryKeyPage(
                        recoveryKey: extra is RecoveryKeyText ? extra : null,
                      ),
                    );
                  },
                ),
              ],
            ),
            GoRoute(
              path: KansoRoutes.import,
              pageBuilder: (context, state) {
                return CupertinoPage<void>(
                  key: state.pageKey,
                  child: ImportPage(
                    path: state
                        .uri
                        .queryParameters[KansoRoutes.importPathParameter],
                    kind: state
                        .uri
                        .queryParameters[KansoRoutes.importKindParameter],
                  ),
                );
              },
            ),
            GoRoute(
              path: KansoRoutes.shared,
              pageBuilder: (context, state) => CupertinoPage<void>(
                key: state.pageKey,
                child: const SharedWithYouPage(),
              ),
              routes: <RouteBase>[
                GoRoute(
                  path: 'join',
                  pageBuilder: (context, state) => CupertinoPage<void>(
                    key: state.pageKey,
                    child: const JoinLinkPage(),
                  ),
                ),
              ],
            ),
          ],
        ),
        if (_gallery.available)
          GoRoute(
            path: KansoRoutes.design,
            pageBuilder: (context, state) => CupertinoPage<void>(
              key: state.pageKey,
              child: const DesignGalleryPage(),
            ),
          ),
      ],
    );
  }

  GoRoute _placeRoute(String path, ListPlace Function(GoRouterState) place) {
    return GoRoute(
      path: path,
      pageBuilder: (context, state) =>
          _listPage(state, single: NoteListPage(place: place(state))),
      routes: <RouteBase>[_noteRoute()],
    );
  }

  GoRoute _noteRoute() {
    return GoRoute(
      path: '${KansoRoutes.note}/:noteId',
      pageBuilder: (context, state) {
        final noteId = state.pathParameters['noteId']!;
        return layoutClass.value == LayoutClass.single
            ? CupertinoPage<void>(
                key: state.pageKey,
                child: NotePage(noteId: noteId),
              )
            : NoTransitionPage<void>(
                key: state.pageKey,
                child: NotePage(noteId: noteId, inPane: true),
              );
      },
    );
  }

  /// The page of a place or a search itself. In one pane it is the list,
  /// [single]; in more the list is beside the note (see `BrowseShell`), and
  /// this page, in the note pane, says that no note is open.
  Page<void> _listPage(GoRouterState state, {required Widget single}) {
    return layoutClass.value == LayoutClass.single
        ? CupertinoPage<void>(key: state.pageKey, child: single)
        : NoTransitionPage<void>(
            key: state.pageKey,
            child: const NoNoteSelectedPane(),
          );
  }
}
