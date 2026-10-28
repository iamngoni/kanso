// Kanso — daily_template_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/templates_repository.dart';

part 'daily_template_bloc.freezed.dart';
part 'daily_template_event.dart';
part 'daily_template_state.dart';

/// The template new daily notes start from (the desktop's Settings >
/// Daily notes > Daily template): the one chosen, if any, and the
/// templates to choose from.
@injectable
class DailyTemplateBloc extends Bloc<DailyTemplateEvent, DailyTemplateState> {
  DailyTemplateBloc(this._templates) : super(const DailyTemplateState()) {
    on<DailyTemplateStartedEvent>(_onStarted, transformer: restartable());
    on<DailyTemplateChosenEvent>(_onChosen, transformer: sequential());
    on<DailyTemplateErrorDismissedEvent>(
      (event, emit) => emit(state.copyWith(error: null)),
    );
  }

  final TemplatesRepository _templates;

  Future<void> _onStarted(
    DailyTemplateStartedEvent event,
    Emitter<DailyTemplateState> emit,
  ) async {
    final results = await (
      _templates.templates(),
      _templates.dailyTemplate(),
    ).wait;
    if (emit.isDone) {
      return;
    }
    final (templates, current) = results;
    templates.fold(
      (error) => emit(
        state.copyWith(status: LoadStatus.failed, error: error),
      ),
      (rows) => current.fold(
        (error) => emit(
          state.copyWith(status: LoadStatus.failed, error: error),
        ),
        (id) => emit(
          DailyTemplateState(
            status: LoadStatus.ready,
            templates: rows,
            // A template that is gone is no template.
            current: rows.any((row) => row.id == id) ? id : null,
          ),
        ),
      ),
    );
  }

  Future<void> _onChosen(
    DailyTemplateChosenEvent event,
    Emitter<DailyTemplateState> emit,
  ) async {
    final result = await _templates.setDailyTemplate(event.templateId);
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(error: error)),
      (_) => emit(state.copyWith(current: event.templateId, error: null)),
    );
  }
}
