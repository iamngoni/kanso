// Kanso: opened_documents_host.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../state/opened_documents/opened_documents_bloc.dart';
import 'opened_documents_notice.dart';

/// [child], and what comes of documents other apps open in Kanso: the note
/// the last one became is shown (in All Notes, over whatever the person was
/// doing), and a notice over a veil says what could not be opened. It needs
/// an [OpenedDocumentsBloc] above it; [router] is the app's, since this sits
/// above the navigator.
class OpenedDocumentsHost extends StatelessWidget {
  const OpenedDocumentsHost({
    required this.router,
    required this.child,
    super.key,
  });

  final GoRouter router;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    if (Provider.of<OpenedDocumentsBloc?>(context, listen: false) == null) {
      return child;
    }
    return BlocConsumer<OpenedDocumentsBloc, OpenedDocumentsState>(
      listenWhen: (before, now) =>
          now.noteId != null && before.noteId != now.noteId,
      listener: (context, state) {
        router.go(
          const BrowseLocation.place(
            ListPlace.all(),
          ).withNote(state.noteId!).location,
        );
        if (!state.hasNotice) {
          context.read<OpenedDocumentsBloc>().add(
            const OpenedDocumentsEvent.dismissed(),
          );
        }
      },
      builder: (context, state) {
        return Stack(
          children: <Widget>[
            Positioned.fill(child: child),
            if (state.hasNotice) ...<Widget>[
              const Positioned.fill(child: AbsorbPointer(child: KansoScrim())),
              Positioned.fill(
                // Above the app's navigator there is no page to give the
                // text its style, so the notice sets plain text itself.
                child: KansoPlainText(
                  child: Center(
                    child: SingleChildScrollView(
                      child: Padding(
                        padding: const EdgeInsets.all(KansoSpacing.lg),
                        child: OpenedDocumentsNotice(
                          problems: state.problems,
                          failed: state.failed,
                          onDismiss: () => context
                              .read<OpenedDocumentsBloc>()
                              .add(const OpenedDocumentsEvent.dismissed()),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ],
        );
      },
    );
  }
}
