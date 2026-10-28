// Kanso — icon_themes_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart' show IconThemeRow;
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';

part 'icon_themes_bloc.freezed.dart';
part 'icon_themes_event.dart';
part 'icon_themes_state.dart';

/// The icon themes the Icons page offers: the built-in ones in the desktop's
/// order, each with a few of its icons.
///
/// Which is in use, and picking one, are the `AppearanceBloc`'s.
@injectable
class IconThemesBloc extends Bloc<IconThemesEvent, IconThemesState> {
  IconThemesBloc(this._renderer) : super(const IconThemesState()) {
    on<IconThemesStartedEvent>(_onStarted, transformer: restartable());
  }

  final RendererRepository _renderer;

  Future<void> _onStarted(
    IconThemesStartedEvent event,
    Emitter<IconThemesState> emit,
  ) async {
    emit(const IconThemesState());
    final result = await _renderer.iconThemes();
    emit(
      result.fold(
        (error) => const IconThemesState(status: LoadStatus.failed),
        (themes) => IconThemesState(status: LoadStatus.ready, themes: themes),
      ),
    );
  }
}
