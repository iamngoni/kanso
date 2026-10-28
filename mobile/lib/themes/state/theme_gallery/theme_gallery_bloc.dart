// Kanso — theme_gallery_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/gallery_repository.dart';
import '../../models/data/gallery_filter.dart';
import '../../models/data/gallery_problem.dart';
import '../../utils/gallery_items_shown.dart';
import '../../utils/gallery_problem_of.dart';
import '../../utils/installed_color_theme_ids.dart';

part 'theme_gallery_bloc.freezed.dart';
part 'theme_gallery_event.dart';
part 'theme_gallery_state.dart';

/// The theme gallery on the theme picker: the community's themes to search,
/// order, install, update and remove.
///
/// The index is verified by the Rust core exactly as the desktop does. The
/// last one fetched is shown at once, and the current one is fetched behind
/// it; when that cannot be, the last one stays with a note that it may be
/// old. Installing, updating and removing go one after another.
@injectable
class ThemeGalleryBloc extends Bloc<ThemeGalleryEvent, ThemeGalleryState> {
  ThemeGalleryBloc(this._gallery) : super(const ThemeGalleryState()) {
    on<ThemeGalleryStartedEvent>(_onStarted, transformer: restartable());
    on<ThemeGalleryRefreshedEvent>(_onRefreshed, transformer: restartable());
    on<ThemeGallerySearchedEvent>(
      (event, emit) => emit(state.copyWith(query: event.query)),
    );
    on<ThemeGalleryFilterChosenEvent>(
      (event, emit) => emit(state.copyWith(filter: event.filter)),
    );
    on<ThemeGalleryInstalledEvent>(
      (event, emit) => _change(event.id, _gallery.install, emit),
      transformer: sequential(),
    );
    on<ThemeGalleryUninstalledEvent>(
      (event, emit) => _change(event.id, _gallery.uninstall, emit),
      transformer: sequential(),
    );
  }

  final GalleryRepository _gallery;

  Future<void> _onStarted(
    ThemeGalleryStartedEvent event,
    Emitter<ThemeGalleryState> emit,
  ) async {
    emit(ThemeGalleryState(query: state.query, filter: state.filter));
    final cached = await _gallery.index();
    final shown = cached.fold<List<GalleryItem>?>(
      (_) => null,
      (items) => items,
    );
    if (shown == null) {
      emit(state.copyWith(status: LoadStatus.failed));
      return;
    }
    emit(
      state.copyWith(status: LoadStatus.ready, items: shown, refreshing: true),
    );
    await _refresh(emit);
  }

  Future<void> _onRefreshed(
    ThemeGalleryRefreshedEvent event,
    Emitter<ThemeGalleryState> emit,
  ) async {
    if (state.status != LoadStatus.ready) {
      return;
    }
    emit(state.copyWith(refreshing: true, refreshFailed: false));
    await _refresh(emit);
  }

  /// Fetches the current index behind the one shown.
  Future<void> _refresh(Emitter<ThemeGalleryState> emit) async {
    final fetched = await _gallery.index(refresh: true);
    emit(
      fetched.fold(
        (error) => state.copyWith(refreshing: false, refreshFailed: true),
        (items) => _withItems(
          state.copyWith(refreshing: false, refreshFailed: false),
          items,
          // A theme the catalog withdrew is removed by this fetch.
          touched: <String>{},
        ),
      ),
    );
  }

  Future<void> _change(
    String id,
    Future<Either<ApplicationError, List<GalleryItem>>> Function(String id)
    change,
    Emitter<ThemeGalleryState> emit,
  ) async {
    emit(
      state.copyWith(working: <String>{...state.working, id}, problem: null),
    );
    final changed = await change(id);
    final working = <String>{...state.working}..remove(id);
    emit(
      changed.fold(
        (error) =>
            state.copyWith(working: working, problem: galleryProblemOf(error)),
        (items) => _withItems(
          state.copyWith(working: working),
          items,
          touched: <String>{id},
        ),
      ),
    );
  }

  /// [from] showing [items], and if the themes installed from the gallery
  /// are not the same now (or one of [touched] was installed again), a note
  /// of which changed for whoever listens.
  ThemeGalleryState _withItems(
    ThemeGalleryState from,
    List<GalleryItem> items, {
    required Set<String> touched,
  }) {
    final before = installedColorThemeIds(from.items);
    final after = installedColorThemeIds(items);
    final changed = <String>{
      ...before.difference(after),
      ...after.difference(before),
      ...touched,
    };
    return from.copyWith(
      items: items,
      changed: changed,
      installedRevision: changed.isEmpty
          ? from.installedRevision
          : from.installedRevision + 1,
    );
  }
}
