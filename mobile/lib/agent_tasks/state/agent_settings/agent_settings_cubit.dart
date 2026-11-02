// Kanso — agent_settings_cubit.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/agent_task_prefs.dart';
import '../../../core/models/repos/abs/agent_tasks_repository.dart';

/// Settings › AI › Agent tasks. A change shows at once and is saved; if the
/// save fails the saved settings come back.
@injectable
class AgentSettingsCubit extends Cubit<AgentTaskPrefs> {
  AgentSettingsCubit(this._tasks) : super(const AgentTaskPrefs());

  final AgentTasksRepository _tasks;

  Future<void> load() async {
    (await _tasks.prefs()).fold((_) {}, emit);
  }

  Future<void> change(AgentTaskPrefs prefs) async {
    final before = state;
    emit(prefs);
    final saved = await _tasks.savePrefs(prefs);
    saved.fold((_) => emit(before), emit);
  }
}
