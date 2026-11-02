// Kanso — agent_tasks_library_row.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';

/// The library's Agent task row, drawn only while AI is on.
class AgentTasksLibraryRow extends StatelessWidget {
  const AgentTasksLibraryRow({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AiStatusBloc, AiStatusState>(
      buildWhen: (before, now) => before.enabled != now.enabled,
      builder: (context, state) {
        if (!state.enabled) {
          return const SizedBox.shrink();
        }
        return KansoSidebarRow(
          icon: KansoIcons.listChecks,
          label: context.l10n.agentTasksEntry,
          onTap: () {
            KansoDrawerScope.maybeOf(context)?.close();
            context.push(KansoRoutes.agentTasks);
          },
        );
      },
    );
  }
}
