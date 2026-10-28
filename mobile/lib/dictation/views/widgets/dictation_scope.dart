// Kanso — dictation_scope.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../editor_extras/models/data/editor_extras.dart';
import '../../../editor_extras/views/widgets/editor_extras_scope.dart';
import '../../../injection.dart';
import '../../models/repos/abs/dictation_text_rules.dart';
import '../../state/dictation/dictation_bloc.dart';
import '../../utils/combined_editor_extras.dart';
import '../../utils/dictation_editor_extras.dart';
import 'dictation_overlay.dart';

/// Puts dictation in reach of the editor below it: the key in the
/// accessory bar (through [EditorExtrasScope]) and the pill that shows above
/// the bar while it listens.
///
/// Place it once, inside the `MaterialApp` (its `builder`), so that it has
/// the theme and the strings and sits over every page. The editor reads one
/// `EditorExtras`: pass the others' (the inline assistant's) as [others],
/// and dictation's key comes after theirs.
class DictationScope extends StatefulWidget {
  const DictationScope({
    required this.child,
    this.others,
    this.rules,
    this.bloc,
    super.key,
  });

  final Widget child;

  /// The extras of other features, whose keys and actions come first.
  final EditorExtras? others;

  /// How spoken words become text; the app's when null (tests give theirs).
  final DictationTextRules? rules;

  /// The BLoC to dictate with; one is made when null (tests give theirs).
  final DictationBloc? bloc;

  @override
  State<DictationScope> createState() => _DictationScopeState();
}

class _DictationScopeState extends State<DictationScope> {
  late final DictationBloc bloc = widget.bloc ?? getIt<DictationBloc>();
  late final DictationEditorExtras extras = DictationEditorExtras(
    bloc: bloc,
    rules: widget.rules ?? getIt<DictationTextRules>(),
  );

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    extras.label = context.l10n.dictationKey;
  }

  @override
  void dispose() {
    extras.dispose();
    if (widget.bloc == null) {
      bloc.close();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<DictationBloc>.value(
      value: bloc,
      child: EditorExtrasScope(
        extras: widget.others == null
            ? extras
            : CombinedEditorExtras(widget.others!, extras),
        child: Stack(
          fit: StackFit.expand,
          children: <Widget>[
            widget.child,
            DictationOverlay(preview: extras.preview),
          ],
        ),
      ),
    );
  }
}
