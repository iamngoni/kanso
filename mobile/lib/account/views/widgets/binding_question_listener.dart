// Kanso — binding_question_listener.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import 'binding_question_sheet.dart';

/// Asks the question about the notes on this phone, in a sheet, whenever the
/// BLoC `B` comes to hold one, and tells [onAnswered] the answer (null for
/// Cancel).
class BindingQuestionListener<B extends StateStreamable<S>, S>
    extends StatelessWidget {
  const BindingQuestionListener({
    required this.questionOf,
    required this.onAnswered,
    required this.child,
    super.key,
  });

  /// The question a state holds, if it holds one.
  final BindingQuestion? Function(S state) questionOf;

  /// What the answer is told to.
  final void Function(BuildContext context, BindingAnswer? answer) onAnswered;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<B, S>(
      listenWhen: (before, after) =>
          questionOf(before) == null && questionOf(after) != null,
      listener: (context, state) async {
        final question = questionOf(state);
        if (question == null) {
          return;
        }
        final answer = await BindingQuestionSheet.show(context, question);
        if (context.mounted) {
          onAnswered(context, answer);
        }
      },
      child: child,
    );
  }
}
