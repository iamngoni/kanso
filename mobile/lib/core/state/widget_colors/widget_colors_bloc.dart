// Kanso — widget_colors_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../configs/kanso_colors.dart';
import '../../models/data/application_error.dart';
import '../../models/repos/abs/widget_colors_repository.dart';

part 'widget_colors_bloc.freezed.dart';
part 'widget_colors_event.dart';
part 'widget_colors_state.dart';

/// Keeps the home-screen widgets in the colours of the app's themes: told
/// the light and dark colours at the start and whenever they change, it
/// writes them for the widgets to read and has the widgets redrawn.
///
/// Changes are handled one after another, in order, so the last one wins.
@injectable
class WidgetColorsBloc extends Bloc<WidgetColorsEvent, WidgetColorsState> {
  WidgetColorsBloc(this._colors) : super(const WidgetColorsState()) {
    on<WidgetColorsChangedEvent>(_onChanged, transformer: sequential());
  }

  final WidgetColorsRepository _colors;

  Future<void> _onChanged(
    WidgetColorsChangedEvent event,
    Emitter<WidgetColorsState> emit,
  ) async {
    final result = await _colors.publish(
      light: event.light,
      dark: event.dark,
    );
    emit(
      result.fold(
        (error) => state.copyWith(error: error),
        (written) => state.copyWith(
          error: null,
          publishes: written ? state.publishes + 1 : state.publishes,
        ),
      ),
    );
  }
}
