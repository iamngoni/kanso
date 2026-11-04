// Kanso — send_feedback_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/send_feedback/send_feedback_bloc.dart';
import '../widgets/send_feedback_pane.dart';

/// Settings › Feedback › Send feedback (`/settings/feedback/new`).
class SendFeedbackPage extends StatelessWidget {
  const SendFeedbackPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SendFeedbackBloc>(
      create: (context) =>
          getIt<SendFeedbackBloc>()..add(const SendFeedbackEvent.started()),
      child: KansoPage(
        backgroundColor: context.colors.app,
        body: const SendFeedbackPane(),
      ),
    );
  }
}
