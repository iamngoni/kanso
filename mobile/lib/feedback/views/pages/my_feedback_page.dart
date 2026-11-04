// Kanso — my_feedback_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/my_feedback/my_feedback_bloc.dart';
import '../widgets/my_feedback_pane.dart';

/// Settings › Feedback › My feedback (`/settings/feedback`). Opening it asks
/// the server how the reports stand, once.
class MyFeedbackPage extends StatelessWidget {
  const MyFeedbackPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<MyFeedbackBloc>(
      create: (context) =>
          getIt<MyFeedbackBloc>()
            ..add(const MyFeedbackEvent.refreshRequested()),
      child: KansoPage(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const MyFeedbackPane(),
      ),
    );
  }
}
