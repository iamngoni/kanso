// Kanso — kanso_routes.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The paths of the app's screens and the names of their parameters.
///
/// Places and searches are written by `BrowseLocation`; this has the rest.
abstract final class KansoRoutes {
  /// `/`: redirects to [notes].
  static const String home = '/';

  /// All Notes.
  static const String notes = '/notes';
  static const String pinned = '/pinned';
  static const String recent = '/recent';
  static const String trash = '/trash';

  /// Tasks, across the notes.
  static const String tasks = '/tasks';

  /// A notebook is `/notebooks/:notebookId`.
  static const String notebooks = '/notebooks';

  /// A tag is `/tags/:tagId`.
  static const String tags = '/tags';

  /// A search is `/search?q=&scope=&in=`.
  static const String search = '/search';

  /// A note opened from a place or a search is that address plus
  /// `/note/:noteId`.
  static const String note = 'note';

  /// The board of a place's notes is that address plus `/board?kind=`.
  static const String board = 'board';

  /// The zero-based source line to focus when opening a task reminder.
  static const String taskLineParameter = 'line';

  /// The UTF-16 body caret for a note opened from a template.
  static const String madeCursorParameter = 'cursor';

  /// Whether a newly made note should focus its title.
  static const String madeTitleFirstParameter = 'titleFirst';

  /// A note by its id alone: redirects to the note in All Notes.
  static const String noteById = '/note';

  /// A notebook by its id alone, as a link names it: redirects to
  /// [notebooks].
  static const String notebookById = '/notebook';

  /// Today's daily note, made if it isn't there: `kanso://open/daily`. It
  /// opens the note in All Notes.
  static const String daily = '/daily';

  /// A new voice note made from speech.
  static const String voiceNote = '/voice-note';

  /// A new note in Edit, as the + button makes: `kanso://open/new`. With
  /// `?voice=1` it starts in its text, to dictate into.
  static const String newNote = '/new';

  /// The parameter of [newNote] that makes it a voice note.
  static const String voiceParameter = 'voice';

  /// Ask the notes a question, a page on a phone and sheet on a tablet.
  static const String ask = '/ask';

  /// Settings, a sheet on a tablet and a page on a phone.
  static const String settings = '/settings';
  static const String settingsAi = '/settings/ai';
  static const String settingsVoice = '/settings/voice';
  static const String settingsNotebook = '/settings/notebook';
  static const String settingsTheme = '/settings/theme';
  static const String settingsIcons = '/settings/icons';
  static const String settingsFonts = '/settings/fonts';
  static const String settingsIconThemes = '/settings/icon-themes';
  static const String settingsLicences = '/settings/licences';
  static const String settingsCrashReports = '/settings/crash-reports';
  static const String settingsBackups = '/settings/backups';

  /// Shared with you, the notes other people shared with this account: a
  /// sheet on a tablet and a page on a phone, like Settings.
  static const String shared = '/shared';

  /// Join with a link, inside [shared].
  static const String sharedJoin = '/shared/join';

  /// Import notes, in the same shell as Settings. With `path` it reads that
  /// file or folder instead of asking for one (see [importFile]).
  static const String import = '/import';

  /// The query parameter that gives the file or folder to import.
  static const String importPathParameter = 'path';

  /// The query parameter that says which source the file is (one of the
  /// names `import_sources` lists), when that is known.
  static const String importKindParameter = 'kind';

  /// The address that imports the file or folder at [path] (a readable local
  /// path that stays where it is until the flow ends), as a [kind] when it
  /// is known. With one possible source the flow goes straight to what the
  /// file holds; with several it asks which; with none it says the file is
  /// not an export.
  static String importFile(String path, {String? kind}) {
    return Uri(
      path: import,
      queryParameters: <String, String>{
        importPathParameter: path,
        if (kind != null && kind.isNotEmpty) importKindParameter: kind,
      },
    ).toString();
  }

  /// A note's saved versions: `/history/:noteId`.
  static const String history = '/history';

  /// Replace text across the library.
  static const String replace = '/replace';

  /// The linked-note graph, optionally centered on a note.
  static const String graph = '/graph';

  /// Opens the graph around [noteId].
  static String graphLocal(String noteId) => Uri(
    path: graph,
    queryParameters: <String, String>{'center': noteId, 'hops': '2'},
  ).toString();

  /// Settings › Account: the signed-in account's page.
  static const String settingsAccount = '/settings/account';

  /// Signing in and making an account.
  static const String settingsSignIn = '/settings/account/sign-in';

  /// Opening the notes of an account that is signed in on a phone with no
  /// key for them yet.
  static const String settingsUnlock = '/settings/account/unlock';

  /// Getting into an account with its recovery key.
  static const String settingsRecover = '/settings/account/recover';

  /// The recovery key to write down.
  static const String settingsRecoveryKey = '/settings/account/recovery-key';

  /// The design gallery: in the development environment only.
  static const String design = '/design';

  /// The query parameter that opens Fonts at its text size section.
  static const String fontsAtParameter = 'at';

  /// The value of [fontsAtParameter] for the text size section.
  static const String fontsAtSize = 'size';

  /// The query parameter that names the server the recovery form is for.
  static const String serverParameter = 'server';

  /// The address of the recovery form, for the account on [server] (the
  /// default server when none is given).
  static String recover({String? server}) {
    if (server == null || server.isEmpty) {
      return settingsRecover;
    }
    return Uri(
      path: settingsRecover,
      queryParameters: <String, String>{serverParameter: server},
    ).toString();
  }

  /// The address of Fonts, opened at the text size section if [atSize].
  static String fonts({bool atSize = false}) =>
      atSize ? '$settingsFonts?$fontsAtParameter=$fontsAtSize' : settingsFonts;
}
