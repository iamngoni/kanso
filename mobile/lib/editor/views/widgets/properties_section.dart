// Kanso — properties_section.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import 'new_property_row.dart';
import 'properties_row.dart';
import 'property_row.dart';

/// A note's properties above its text: the row with how many there are,
/// which opens the list to change them, take one away or add one.
class PropertiesSection extends StatefulWidget {
  const PropertiesSection({
    required this.properties,
    required this.open,
    required this.onToggle,
    required this.onSet,
    required this.onRemove,
    required this.onPick,
    required this.loadZones,
    this.failed = false,
    this.dropped = false,
    this.readOnly = false,
    super.key,
  });

  final List<Property> properties;
  final bool open;

  /// The row was tapped.
  final VoidCallback onToggle;

  /// The value of the property in a row was changed (that row is `index`),
  /// or, with no index, the property called `key` was set (added when there
  /// is none).
  final void Function(String key, String value, {int? index}) onSet;

  /// The property in the row at this index was asked to go.
  final ValueChanged<int> onRemove;

  /// A picker picked a choice for the property in row `index`, which it
  /// showed as `key` holding `original`.
  final void Function(
    int index,
    String key,
    String original,
    PropertyChoice choice,
  )
  onPick;

  /// Every time zone, for the zone picker.
  final Future<List<ZoneRow>> Function() loadZones;

  /// Whether the last change did not work.
  final bool failed;

  /// Whether the last pick was dropped because the properties changed under
  /// it.
  final bool dropped;

  /// Whether the properties may be read and not changed: they open and
  /// close, and nothing is added.
  final bool readOnly;

  @override
  State<PropertiesSection> createState() => _PropertiesSectionState();
}

class _PropertiesSectionState extends State<PropertiesSection> {
  final ValueNotifier<bool> adding = ValueNotifier<bool>(false);

  @override
  void didUpdateWidget(PropertiesSection oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.open && !widget.open) {
      adding.value = false;
    }
  }

  @override
  void dispose() {
    adding.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: PropertiesRow(
            count: widget.properties.length,
            open: widget.open,
            onTap: widget.onToggle,
          ),
        ),
        if (widget.open) ...<Widget>[
          for (var i = 0; i < widget.properties.length; i++)
            IgnorePointer(
              // A row is its place in the front matter.
              key: ValueKey<(int, String)>((i, widget.properties[i].key)),
              ignoring: widget.readOnly,
              child: PropertyRow(
                property: widget.properties[i],
                onSubmitted: (value) =>
                    widget.onSet(widget.properties[i].key, value, index: i),
                onRemove: () => widget.onRemove(i),
                onPick: (choice) => widget.onPick(
                  i,
                  widget.properties[i].key,
                  widget.properties[i].text,
                  choice,
                ),
                loadZones: widget.loadZones,
              ),
            ),
          if (widget.dropped)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
              child: Semantics(
                liveRegion: true,
                child: Text(
                  l10n.propertiesDropped,
                  style: context.typography.meta.copyWith(
                    color: colors.warning,
                  ),
                ),
              ),
            ),
          if (widget.failed)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
              child: Semantics(
                liveRegion: true,
                child: Text(
                  l10n.propertiesFailed,
                  style: context.typography.meta.copyWith(color: colors.danger),
                ),
              ),
            ),
          if (!widget.readOnly)
            ValueListenableBuilder<bool>(
              valueListenable: adding,
              builder: (context, isAdding, _) {
                if (isAdding) {
                  return NewPropertyRow(
                    onAdd: (key, value) {
                      adding.value = false;
                      widget.onSet(key, value);
                    },
                    onCancel: () => adding.value = false,
                  );
                }
                return Align(
                  alignment: AlignmentDirectional.centerStart,
                  child: KansoPressable(
                    onTap: () => adding.value = true,
                    child: SizedBox(
                      height: KansoSizes.touchTarget,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: <Widget>[
                          KansoIcon(
                            KansoIcons.plus,
                            size: KansoSizes.iconSmall,
                            color: colors.accent,
                          ),
                          const SizedBox(width: KansoSpacing.xs),
                          Text(
                            l10n.propertiesAdd,
                            style: context.typography.label.copyWith(
                              color: colors.accent,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
        ],
      ],
    );
  }
}
