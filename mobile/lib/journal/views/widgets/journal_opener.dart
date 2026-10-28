// Kanso — journal_opener.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_notice.dart';
import '../../../injection.dart';
import '../../state/journal/journal_bloc.dart';
import '../../utils/open_made_note.dart';

/// Gives what is below it a [JournalBloc], opens the notes it makes, and
/// says why one could not be made.
class JournalOpener extends StatelessWidget {
  const JournalOpener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<JournalBloc>(
      create: (_) => getIt<JournalBloc>(),
      child: BlocListener<JournalBloc, JournalState>(
        listenWhen: (before, after) =>
            after.opened != null && after.opened != before.opened,
        listener: (context, state) => openMadeNote(context, state.opened!),
        // The child is not positioned: a Stack sizes itself to its
        // non-positioned children, and the notice below is a `SizedBox` of
        // nothing most of the time, so a `Positioned.fill` child would be
        // laid out against a zero-size Stack.
        child: Stack(
          fit: StackFit.passthrough,
          children: <Widget>[
            child,
            BlocBuilder<JournalBloc, JournalState>(
              buildWhen: (before, after) => before.error != after.error,
              builder: (context, state) {
                if (state.error == null) {
                  return const SizedBox.shrink();
                }
                return PositionedDirectional(
                  start: KansoSpacing.md,
                  end: KansoSpacing.md,
                  bottom:
                      MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
                  child: KansoNotice(
                    message: context.l10n.journalFailed,
                    dismissLabel: context.l10n.noteListDismiss,
                    onDismiss: () => context.read<JournalBloc>().add(
                      const JournalEvent.errorDismissed(),
                    ),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}
