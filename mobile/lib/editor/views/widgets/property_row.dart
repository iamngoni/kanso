// Kanso — property_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_text_input.dart';

/// One property of the note: its name, its value to change, and a button to
/// remove it.
///
/// A yes or no is a switch; any other value is a line of text in the form the
/// core wrote it (a list is its items with commas between) that is set when
/// the return key is pressed or the field is left.
class PropertyRow extends StatefulWidget {
  const PropertyRow({
    required this.property,
    required this.onSubmitted,
    required this.onRemove,
    super.key,
  });

  final Property property;

  /// The value was changed to this.
  final ValueChanged<String> onSubmitted;

  /// The property was asked to go.
  final VoidCallback onRemove;

  @override
  State<PropertyRow> createState() => _PropertyRowState();
}

class _PropertyRowState extends State<PropertyRow> {
  late final TextEditingController controller;
  final FocusNode focusNode = FocusNode();

  /// The value last passed on, so that the return key and then leaving the
  /// field do not set it twice.
  String? sent;

  @override
  void initState() {
    super.initState();
    controller = TextEditingController(text: widget.property.text);
    focusNode.addListener(left);
  }

  @override
  void didUpdateWidget(PropertyRow oldWidget) {
    super.didUpdateWidget(oldWidget);
    // What the core says is the value, unless it is being typed.
    if (oldWidget.property.text != widget.property.text) {
      sent = null;
      if (!focusNode.hasFocus) {
        controller.text = widget.property.text;
      }
    }
  }

  /// Sets the value when the field is left with a different one.
  void left() {
    if (!focusNode.hasFocus) {
      submit(controller.text);
    }
  }

  void submit(String value) {
    if (value != widget.property.text && value != sent) {
      sent = value;
      widget.onSubmitted(value);
    }
  }

  @override
  void dispose() {
    focusNode
      ..removeListener(left)
      ..dispose();
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final property = widget.property;
    final field = property.kind == PropertyKind.bool
        ? Align(
            alignment: AlignmentDirectional.centerStart,
            child: KansoSwitch(
              value: property.text == 'true',
              onChanged: (on) => widget.onSubmitted(on ? 'true' : 'false'),
            ),
          )
        : KansoTextInput(
            controller: controller,
            focusNode: focusNode,
            hint: l10n.propertiesValue,
            semanticLabel: property.key,
            keyboardType: property.kind == PropertyKind.number
                ? const TextInputType.numberWithOptions(
                    signed: true,
                    decimal: true,
                  )
                : TextInputType.text,
            autocorrect: false,
            onSubmitted: submit,
          );
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
      child: Row(
        children: <Widget>[
          Expanded(
            flex: 2,
            child: Text(
              property.key,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.label.copyWith(
                color: colors.textSecondary,
              ),
            ),
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(flex: 3, child: field),
          KansoIconButton(
            icon: KansoIcons.close,
            semanticLabel: l10n.propertiesRemove(property.key),
            onPressed: widget.onRemove,
          ),
        ],
      ),
    );
  }
}
