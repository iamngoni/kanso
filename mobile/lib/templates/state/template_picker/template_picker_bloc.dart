// Kanso — template_picker_bloc.dart
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

part 'template_picker_bloc.freezed.dart';
part 'template_picker_event.dart';
part 'template_picker_state.dart';

/// The templates to choose from, for "New from template" and for the
/// daily template: the notes of the Templates notebook, in the library's
/// order.
@injectable
class TemplatePickerBloc
    extends Bloc<TemplatePickerEvent, TemplatePickerState> {
  TemplatePickerBloc(this._templates) : super(const TemplatePickerState()) {
    on<TemplatePickerStartedEvent>(_onStarted, transformer: restartable());
  }

  final TemplatesRepository _templates;

  Future<void> _onStarted(
    TemplatePickerStartedEvent event,
    Emitter<TemplatePickerState> emit,
  ) async {
    emit(const TemplatePickerState());
    final result = await _templates.templates();
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(
        TemplatePickerState(status: LoadStatus.failed, error: error),
      ),
      (rows) => emit(
        TemplatePickerState(status: LoadStatus.ready, templates: rows),
      ),
    );
  }
}
