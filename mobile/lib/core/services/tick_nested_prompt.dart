// Kanso — tick_nested_prompt.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';

import '../../injection.dart';
import '../router/kanso_router.dart';
import '../utils/context_extensions.dart';
import '../views/widgets/kanso_dialog.dart';

/// The question put the first time ticking a task would also tick the tasks
/// nested under it: "Tick the tasks under it too?".
// ignore: one_member_abstracts
abstract interface class TickNestedPrompt {
  /// True when the person answers Always; false for Not now (or when there
  /// is no screen to ask on).
  Future<bool> ask();
}

/// Asks in a dialog over the whole app.
@LazySingleton(as: TickNestedPrompt)
class DialogTickNestedPrompt implements TickNestedPrompt {
  @override
  Future<bool> ask() async {
    final context = getIt<KansoRouter>().rootContext;
    if (context == null || !context.mounted) {
      return false;
    }
    final l10n = context.l10n;
    return showKansoConfirm(
      context,
      title: l10n.tickNestedQuestionTitle,
      body: l10n.tickNestedQuestionBody,
      cancelLabel: l10n.tickNestedNotNow,
      confirmLabel: l10n.tickNestedAlways,
    );
  }
}
