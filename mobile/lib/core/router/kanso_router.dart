// Kanso — kanso_router.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';
import 'package:injectable/injectable.dart';

import '../../account/views/pages/account_page.dart';
import '../../account/views/pages/recover_page.dart';
import '../../account/views/pages/recovery_key_page.dart';
import '../../account/views/pages/sign_in_page.dart';
import '../../account/views/pages/unlock_page.dart';
import '../../agent_tasks/views/pages/agent_tasks_page.dart';
import '../../ai_settings/views/pages/ai_settings_page.dart';
import '../../ask/views/pages/ask_page.dart';
import '../../backups/views/pages/backups_page.dart';
import '../../board/views/pages/board_page.dart';
import '../../canvas/views/pages/note_or_canvas_page.dart';
import '../../capture/views/pages/daily_note_page.dart';
import '../../capture/views/pages/new_note_page.dart';
import '../../capture/views/widgets/task_capture_sheet.dart';
import '../../crash/views/pages/crash_reports_page.dart';
import '../../design/views/pages/design_gallery_page.dart';
import '../../dictation/views/pages/voice_note_page.dart';
import '../../digest/views/pages/open_loops_page.dart';
import '../../draft/views/pages/draft_page.dart';
import '../../feedback/views/pages/my_feedback_page.dart';
import '../../feedback/views/pages/send_feedback_page.dart';
import '../../flashcards/views/pages/review_page.dart';
import '../../flashcards/views/pages/review_settings_page.dart';
import '../../graph/views/pages/graph_page.dart';
import '../../history/views/pages/history_page.dart';
import '../../import/views/pages/import_page.dart';
import '../../injection.dart';
import '../../note/views/widgets/no_note_selected_pane.dart';
import '../../note_list/views/pages/note_list_page.dart';
import '../../notebook_home/views/pages/notebook_home_page.dart';
import '../../overview/views/pages/overview_page.dart';
import '../../people/views/pages/people_list_page.dart';
import '../../people/views/pages/people_settings_page.dart';
import '../../recording/views/pages/recording_page.dart';
import '../../replace/views/pages/replace_page.dart';
import '../../schedule/views/pages/schedule_page.dart';
import '../../search/views/pages/search_page.dart';
import '../../settings/views/pages/default_notebook_page.dart';
import '../../settings/views/pages/fonts_page.dart';
import '../../settings/views/pages/icon_themes_page.dart' as icon_picker;
import '../../settings/views/pages/licences_page.dart';
import '../../settings/views/pages/settings_category_pages.dart';
import '../../settings/views/pages/settings_page.dart';
import '../../settings/views/pages/system_calendar_settings_page.dart';
import '../../sharing/views/pages/join_link_page.dart';
import '../../sharing/views/pages/shared_with_you_page.dart';
import '../../shell/views/widgets/browse_shell.dart';
import '../../tasks/views/pages/tasks_page.dart';
import '../../themes/views/pages/icon_themes_page.dart' as icon_themes;
import '../../themes/views/pages/theme_picker_page.dart';
import '../../triage/views/pages/triage_page.dart';
import '../../triage/views/pages/triage_settings_page.dart';
import '../../voice_settings/views/pages/voice_settings_page.dart';
import '../bridge/api/types.dart';
import '../models/data/ask_scope.dart';
import '../models/data/browse_location.dart';
import '../models/data/design_gallery_access.dart';
import '../models/data/layout_class.dart';
import '../models/data/new_link_kind.dart';
import '../models/data/search_scope_kind.dart';
import '../models/repos/abs/digest_repository.dart';
import '../views/widgets/kanso_page_route.dart';
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
///   `/new` makes a note as the + button does (`?voice=1` to dictate into;
///   `?task=1` opens the capture sheet in Task mode over where the app is,
///   and stays there).
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

  /// Whether the first address has been looked at.
  bool _launched = false;

  /// Whether the Task capture sheet is about to be opened.
  bool _taskCapturePending = false;

  /// Opens the capture sheet in Task mode on the root navigator once the
  /// address it stays on has been built; one sheet however many times the
  /// link arrives before then.
  void _openTaskCapture() {
    if (_taskCapturePending) {
      return;
    }
    _taskCapturePending = true;
    _showTaskCaptureWhenReady(_taskCaptureFrames);
  }

  /// How many frames a cold start may take to build the root navigator
  /// before the Task sheet is given up.
  static const int _taskCaptureFrames = 120;

  void _showTaskCaptureWhenReady(int framesLeft) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final context = _rootKey.currentContext;
      if (context == null || !context.mounted) {
        // A cold start: the navigator is not there yet.
        if (framesLeft > 0) {
          _showTaskCaptureWhenReady(framesLeft - 1);
        } else {
          _taskCapturePending = false;
        }
        return;
      }
      _taskCapturePending = false;
      showTaskCaptureSheet(context);
    });
    WidgetsBinding.instance.scheduleFrame();
  }

  GoRouter _build() {
    return GoRouter(
      navigatorKey: _rootKey,
      initialLocation: KansoRoutes.notes,
      refreshListenable: layoutClass,
      // The app's own links carry a scheme and a host, which the router
      // ignores; they are turned into addresses of the app here.
      redirect: (context, state) async {
        if (state.uri.hasScheme) {
          return redirectLocation(state.uri);
        }
        // `kanso://open/new?task=1` (the Add task shortcut and tile) is a
        // sheet, not a page: it opens over where the app already is.
        if (opensTaskCapture(state.uri)) {
          _openTaskCapture();
          final current = config.routerDelegate.currentConfiguration;
          return locationUnderSheet(current.isEmpty ? null : current.uri);
        }
        // The first launch opens to Overview unless Settings says the last
        // note; a link or a notification that names a place is not changed.
        if (!_launched) {
          _launched = true;
          if (state.uri.path == KansoRoutes.notes &&
              state.uri.queryParameters.isEmpty &&
              getIt.isRegistered<DigestRepository>()) {
            final prefs = (await getIt<DigestRepository>().prefs()).fold(
              (_) => null,
              (it) => it,
            );
            if (prefs != null && prefs.openToOverview) {
              return KansoRoutes.overview;
            }
          }
        }
        return null;
      },
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
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: HistoryPage(noteId: state.pathParameters['noteId']!),
          ),
        ),
        GoRoute(
          path: KansoRoutes.graph,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: GraphPage(
              centerId: state.uri.queryParameters['center'],
              bookId: state.uri.queryParameters[KansoRoutes.graphBookParameter],
              hops: int.tryParse(state.uri.queryParameters['hops'] ?? '') ?? 2,
            ),
          ),
        ),
        GoRoute(
          path: KansoRoutes.loops,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const OpenLoopsPage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.overview,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: OverviewPage(
              week:
                  state.uri.queryParameters[KansoRoutes
                      .overviewWeekParameter] ==
                  '1',
            ),
          ),
        ),
        GoRoute(
          path: KansoRoutes.brief,
          redirect: (context, state) => KansoRoutes.overview,
        ),
        GoRoute(
          path: KansoRoutes.review,
          redirect: (context, state) =>
              '${KansoRoutes.overview}?${KansoRoutes.overviewWeekParameter}=1',
        ),
        GoRoute(
          path: KansoRoutes.replace,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const ReplacePage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.record,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: RecordingPage(
              noteId:
                  state.uri.queryParameters[KansoRoutes.recordNoteParameter],
            ),
          ),
        ),
        GoRoute(
          path: KansoRoutes.voiceNote,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const VoiceNotePage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.flashcards,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const FlashcardReviewPage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.flashcardsSettings,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const ReviewSettingsPage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.draft,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: DraftPage(
              noteIds: (state.extra as List<String>?) ?? const <String>[],
            ),
          ),
        ),
        GoRoute(
          path: KansoRoutes.schedule,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const SchedulePage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.people,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const PeopleListPage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.triage,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: const TriagePage(),
          ),
        ),
        GoRoute(
          path: KansoRoutes.agentTasks,
          pageBuilder: (context, state) => kansoPage<void>(
            key: state.pageKey,
            child: AgentTasksPage(
              noteIds: state.extra is List<String>
                  ? state.extra! as List<String>
                  : const <String>[],
            ),
          ),
        ),
        GoRoute(
          path: KansoRoutes.ask,
          pageBuilder: (context, state) {
            final book =
                state.uri.queryParameters[KansoRoutes.askBookParameter];
            final label =
                state.uri.queryParameters[KansoRoutes.askLabelParameter] ?? '';
            final scope = book == null || book.isEmpty
                ? null
                : AskScope(kind: 'book', value: book, label: label);
            return layoutClass.tabletDevice
                ? KansoSheetPage<void>(
                    key: state.pageKey,
                    child: AskPage(inSheet: true, scope: scope),
                  )
                : kansoPage<void>(
                    key: state.pageKey,
                    child: AskPage(scope: scope),
                  );
          },
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
                      newLinkKind(state.uri.queryParameters) ==
                      NewLinkKind.voice,
                ),
              ),
            ),
            _placeRoute(KansoRoutes.notes, (_) => const ListPlace.all()),
            _placeRoute(KansoRoutes.pinned, (_) => const ListPlace.pinned()),
            _placeRoute(KansoRoutes.recent, (_) => const ListPlace.recent()),
            _placeRoute(
              KansoRoutes.canvases,
              (_) => const ListPlace.canvases(),
            ),
            _placeRoute(KansoRoutes.trash, (_) => const ListPlace.trash()),
            _placeRoute(
              '${KansoRoutes.notebooks}/:notebookId',
              (state) =>
                  ListPlace.notebook(id: state.pathParameters['notebookId']!),
              withHome: true,
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
              : kansoPage<void>(key: state.pageKey, child: child),
          routes: <RouteBase>[
            GoRoute(
              path: KansoRoutes.settings,
              pageBuilder: (context, state) => kansoPage<void>(
                key: state.pageKey,
                child: const SettingsPage(),
              ),
              routes: <RouteBase>[
                GoRoute(
                  path: 'general',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const GeneralSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'appearance',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const AppearanceSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'journal',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const JournalSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'calendar',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const SystemCalendarSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'discovery',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const DiscoverySettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'privacy',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const PrivacySettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'about',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const AboutSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'notebook',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const DefaultNotebookPage(),
                  ),
                ),
                GoRoute(
                  path: 'theme',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const ThemePickerPage(),
                  ),
                ),
                GoRoute(
                  path: 'icons',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const icon_picker.IconThemesPage(),
                  ),
                ),
                GoRoute(
                  path: 'icon-themes',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const icon_themes.IconThemesPage(),
                  ),
                ),
                GoRoute(
                  path: 'fonts',
                  pageBuilder: (context, state) => kansoPage<void>(
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
                  path: 'ai',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const AiSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'people',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const PeopleSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'inbox',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const TriageSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'voice',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const VoiceSettingsPage(),
                  ),
                ),
                GoRoute(
                  path: 'licences',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const LicencesPage(),
                  ),
                ),
                GoRoute(
                  path: 'crash-reports',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const CrashReportsPage(),
                  ),
                ),
                GoRoute(
                  path: 'feedback',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const MyFeedbackPage(),
                  ),
                  routes: <RouteBase>[
                    GoRoute(
                      path: 'new',
                      pageBuilder: (context, state) => kansoPage<void>(
                        key: state.pageKey,
                        child: const SendFeedbackPage(),
                      ),
                    ),
                  ],
                ),
                GoRoute(
                  path: 'backups',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const BackupsPage(),
                  ),
                ),
                GoRoute(
                  path: 'account',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const AccountPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/sign-in',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const SignInPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/unlock',
                  pageBuilder: (context, state) => kansoPage<void>(
                    key: state.pageKey,
                    child: const UnlockPage(),
                  ),
                ),
                GoRoute(
                  path: 'account/recover',
                  pageBuilder: (context, state) => kansoPage<void>(
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
                    return kansoPage<void>(
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
                return kansoPage<void>(
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
              pageBuilder: (context, state) => kansoPage<void>(
                key: state.pageKey,
                child: const SharedWithYouPage(),
              ),
              routes: <RouteBase>[
                GoRoute(
                  path: 'join',
                  pageBuilder: (context, state) => kansoPage<void>(
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
            pageBuilder: (context, state) => kansoPage<void>(
              key: state.pageKey,
              child: const DesignGalleryPage(),
            ),
          ),
      ],
    );
  }

  GoRoute _placeRoute(
    String path,
    ListPlace Function(GoRouterState) place, {
    bool withHome = false,
  }) {
    return GoRoute(
      path: path,
      pageBuilder: (context, state) =>
          _listPage(state, single: NoteListPage(place: place(state))),
      routes: <RouteBase>[
        _noteRoute(),
        _boardRoute(place),
        if (withHome) _homeRoute(),
      ],
    );
  }

  /// A notebook's homepage, as its phone page or its tablet note pane.
  GoRoute _homeRoute() {
    return GoRoute(
      path: KansoRoutes.notebookHome,
      pageBuilder: (context, state) {
        final id = state.pathParameters['notebookId']!;
        return layoutClass.value == LayoutClass.single
            ? kansoPage<void>(
                key: state.pageKey,
                child: NotebookHomePage(bookId: id),
              )
            : NoTransitionPage<void>(
                key: state.pageKey,
                child: NotebookHomePage(bookId: id, inPane: true),
              );
      },
    );
  }

  /// The board of a place's notes, as its phone page or its tablet note pane.
  GoRoute _boardRoute(ListPlace Function(GoRouterState) place) {
    return GoRoute(
      path: KansoRoutes.board,
      pageBuilder: (context, state) {
        final kind = state.uri.queryParameters['kind'] == BoardKind.tasks.name
            ? BoardKind.tasks
            : BoardKind.notes;
        final currentPlace = place(state);
        return layoutClass.value == LayoutClass.single
            ? kansoPage<void>(
                key: state.pageKey,
                child: BoardPage(place: currentPlace, kind: kind),
              )
            : NoTransitionPage<void>(
                key: state.pageKey,
                child: BoardPage(
                  place: currentPlace,
                  kind: kind,
                  inPane: true,
                ),
              );
      },
    );
  }

  GoRoute _noteRoute() {
    return GoRoute(
      path: '${KansoRoutes.note}/:noteId',
      pageBuilder: (context, state) {
        final noteId = state.pathParameters['noteId']!;
        final line = _nonNegativeInt(
          state.uri.queryParameters[KansoRoutes.taskLineParameter],
        );
        final lineEnd = _nonNegativeInt(
          state.uri.queryParameters[KansoRoutes.lineEndParameter],
        );
        final cursor16 = _nonNegativeInt(
          state.uri.queryParameters[KansoRoutes.madeCursorParameter],
        );
        final titleFirst =
            state.uri.queryParameters[KansoRoutes.madeTitleFirstParameter] ==
            '1';
        return layoutClass.value == LayoutClass.single
            ? kansoPage<void>(
                key: state.pageKey,
                child: NoteOrCanvasPage(
                  noteId: noteId,
                  bodyLine: line,
                  bodyLineEnd: lineEnd,
                  cursor16: cursor16,
                  titleFirst: titleFirst,
                ),
              )
            : NoTransitionPage<void>(
                key: state.pageKey,
                child: NoteOrCanvasPage(
                  noteId: noteId,
                  inPane: true,
                  bodyLine: line,
                  bodyLineEnd: lineEnd,
                  cursor16: cursor16,
                  titleFirst: titleFirst,
                ),
              );
      },
    );
  }

  int? _nonNegativeInt(String? value) {
    final parsed = int.tryParse(value ?? '');
    return parsed == null || parsed < 0 ? null : parsed;
  }

  /// The page of a place or a search itself. In one pane it is the list,
  /// [single]; in more the list is beside the note (see `BrowseShell`), and
  /// this page, in the note pane, says that no note is open.
  Page<void> _listPage(GoRouterState state, {required Widget single}) {
    return layoutClass.value == LayoutClass.single
        ? kansoPage<void>(key: state.pageKey, child: single)
        : NoTransitionPage<void>(
            key: state.pageKey,
            child: const NoNoteSelectedPane(),
          );
  }
}
