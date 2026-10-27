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

  /// A notebook is `/notebooks/:notebookId`.
  static const String notebooks = '/notebooks';

  /// A tag is `/tags/:tagId`.
  static const String tags = '/tags';

  /// A search is `/search?q=&scope=&in=`.
  static const String search = '/search';

  /// A note opened from a place or a search is that address plus
  /// `/note/:noteId`.
  static const String note = 'note';

  /// A note by its id alone: redirects to the note in All Notes.
  static const String noteById = '/note';

  /// A notebook by its id alone, as a link names it: redirects to
  /// [notebooks].
  static const String notebookById = '/notebook';

  /// Settings, a sheet on a tablet and a page on a phone.
  static const String settings = '/settings';
  static const String settingsNotebook = '/settings/notebook';
  static const String settingsTheme = '/settings/theme';
  static const String settingsFonts = '/settings/fonts';
  static const String settingsLicences = '/settings/licences';
  static const String settingsCrashReports = '/settings/crash-reports';

  /// The design gallery: in the development environment only.
  static const String design = '/design';

  /// The query parameter that opens Fonts at its text size section.
  static const String fontsAtParameter = 'at';

  /// The value of [fontsAtParameter] for the text size section.
  static const String fontsAtSize = 'size';

  /// The address of Fonts, opened at the text size section if [atSize].
  static String fonts({bool atSize = false}) =>
      atSize ? '$settingsFonts?$fontsAtParameter=$fontsAtSize' : settingsFonts;
}
