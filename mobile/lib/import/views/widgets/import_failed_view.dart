// Kanso: import_failed_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/import_problem.dart';
import '../../state/import_flow/import_flow_bloc.dart';
import '../../utils/import_labels.dart';

/// Reading or importing did not finish: what went wrong, in the library's
/// words, with Try again and a way back to the sources. It needs an
/// [ImportFlowBloc] above it.
class ImportFailedView extends StatelessWidget {
  const ImportFailedView({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<ImportFlowBloc, ImportFlowState>(
      builder: (context, state) {
        final bloc = context.read<ImportFlowBloc>();
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: KansoMessageCard(
              icon: KansoIcons.circleX,
              title: l10n.importFailedTitle,
              message: l10n.importProblemText(
                state.problem ?? const ImportProblem.unexpected(),
              ),
              actions: <Widget>[
                KansoButton(
                  label: l10n.importTryAgain,
                  onPressed: () => bloc.add(const ImportFlowEvent.retried()),
                ),
                KansoButton(
                  label: l10n.importChooseAnother,
                  variant: KansoButtonVariant.secondary,
                  onPressed: () => bloc.add(const ImportFlowEvent.restarted()),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
