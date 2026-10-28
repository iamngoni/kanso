// Kanso — theme_gallery_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'theme_gallery_bloc.dart';

@freezed
abstract class ThemeGalleryState with _$ThemeGalleryState {
  const factory ThemeGalleryState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// Every item of the index, in the catalog's order.
    @Default(<GalleryItem>[]) List<GalleryItem> items,

    /// What the search field holds.
    @Default('') String query,
    @Default(GalleryFilter.popular) GalleryFilter filter,

    /// The current index is being fetched while the last one is shown.
    @Default(false) bool refreshing,

    /// The current index could not be fetched; the last one is shown.
    @Default(false) bool refreshFailed,

    /// The items being installed, updated or removed now, by id.
    @Default(<String>{}) Set<String> working,

    /// Why the last install, update or removal did not happen.
    GalleryProblem? problem,

    /// Counts the times the themes installed from the gallery changed, so a
    /// listener can tell that the installed themes must be read again.
    @Default(0) int installedRevision,

    /// The ids of the themes that changed the last time: installed, updated,
    /// removed, or withdrawn by the catalog.
    @Default(<String>{}) Set<String> changed,
  }) = _ThemeGalleryState;

  const ThemeGalleryState._();

  /// The items that match the search, in the order of the filter.
  List<GalleryItem> get shown => galleryItemsShown(items, query, filter);
}
