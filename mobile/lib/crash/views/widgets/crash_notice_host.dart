// Kanso — crash_notice_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import '../../state/crash_reports/crash_reports_bloc.dart';
import 'crash_notice_card.dart';

/// [child] with the crash notice over it, on a veil, while there are crash
/// reports to ask about. Nothing below it can be touched until the person
/// chooses. It needs a [CrashReportsBloc] above it.
class CrashNoticeHost extends StatelessWidget {
  const CrashNoticeHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<CrashReportsBloc, CrashReportsState>(
      builder: (context, state) {
        final bloc = context.read<CrashReportsBloc>();
        return Stack(
          children: <Widget>[
            Positioned.fill(child: child),
            if (state.noticeVisible) ...<Widget>[
              const Positioned.fill(child: AbsorbPointer(child: KansoScrim())),
              Positioned.fill(
                // Above the app's navigator there is no page to give the
                // text its style, so the card sets plain text itself.
                child: KansoPlainText(
                  child: Center(
                    // The text of the reports can make the card taller than a
                    // small window: it scrolls then.
                    child: SingleChildScrollView(
                      child: Padding(
                        padding: const EdgeInsets.all(KansoSpacing.lg),
                        child: CrashNoticeCard(
                          reports: state.reports,
                          showsReport: state.noticeShowsReport,
                          sending: state.sending.isNotEmpty,
                          outcome: state.outcome,
                          onSend: () => bloc.add(
                            const CrashReportsEvent.sendAllRequested(),
                          ),
                          onDiscard: () => bloc.add(
                            const CrashReportsEvent.discardAllRequested(),
                          ),
                          onToggleReport: () => bloc.add(
                            const CrashReportsEvent.noticeReportToggled(),
                          ),
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
