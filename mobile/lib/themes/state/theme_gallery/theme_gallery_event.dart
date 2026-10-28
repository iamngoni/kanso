// Kanso — theme_gallery_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'theme_gallery_bloc.dart';

@freezed
sealed class ThemeGalleryEvent with _$ThemeGalleryEvent {
  /// The picker was opened, or the person asked to try again: show the last
  /// index at once, then fetch the current one.
  const factory ThemeGalleryEvent.started() = ThemeGalleryStartedEvent;

  /// The person asked for the current index again, with a list on screen.
  const factory ThemeGalleryEvent.refreshed() = ThemeGalleryRefreshedEvent;

  /// The search field changed.
  const factory ThemeGalleryEvent.searched(String query) =
      ThemeGallerySearchedEvent;

  /// A filter was chosen.
  const factory ThemeGalleryEvent.filterChosen(GalleryFilter filter) =
      ThemeGalleryFilterChosenEvent;

  /// Install (or Update) was pressed on the item [id].
  const factory ThemeGalleryEvent.installed(String id) =
      ThemeGalleryInstalledEvent;

  /// Remove was pressed on the installed item [id].
  const factory ThemeGalleryEvent.uninstalled(String id) =
      ThemeGalleryUninstalledEvent;
}
