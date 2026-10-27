// Kanso — kanso_field_controllers.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// Owns the [TextEditingController]s of a form's fields and disposes them,
/// and gives them to [builder] in the order of the fields.
///
/// [initialTexts] is what the first fields start with; the rest start empty.
/// The count of fields does not change while the form is on screen.
class KansoFieldControllers extends StatefulWidget {
  const KansoFieldControllers({
    required this.count,
    required this.builder,
    this.initialTexts = const <String>[],
    super.key,
  });

  final int count;
  final List<String> initialTexts;
  final Widget Function(
    BuildContext context,
    List<TextEditingController> controllers,
  )
  builder;

  @override
  State<KansoFieldControllers> createState() => _KansoFieldControllersState();
}

class _KansoFieldControllersState extends State<KansoFieldControllers> {
  late final List<TextEditingController> controllers = <TextEditingController>[
    for (var i = 0; i < widget.count; i++)
      TextEditingController(
        text: i < widget.initialTexts.length ? widget.initialTexts[i] : '',
      ),
  ];

  @override
  void dispose() {
    for (final controller in controllers) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, controllers);
}
