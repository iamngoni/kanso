// Kanso — canvas_card_sheet.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/sheet_header.dart';

/// What the person chose for a card.
sealed class CardChoice {
  const CardChoice();
}

/// Keep this text (a new or an existing text card).
class CardText extends CardChoice {
  const CardText(this.text);
  final String text;
}

class CardDelete extends CardChoice {
  const CardDelete();
}

class CardOpenNote extends CardChoice {
  const CardOpenNote();
}

/// The sheet for one card: a text card's text to write, or a note card's
/// title with a way to open its note. [card] is null for a new text card.
class CanvasCardSheet extends StatefulWidget {
  const CanvasCardSheet({this.card, super.key});

  final CanvasCard? card;

  /// Opens the sheet and returns the choice, or null when it was dismissed.
  static Future<CardChoice?> show(BuildContext context, {CanvasCard? card}) =>
      showKansoBottomSheet<CardChoice>(
        context,
        builder: (_) => CanvasCardSheet(card: card),
      );

  @override
  State<CanvasCardSheet> createState() => _CanvasCardSheetState();
}

class _CanvasCardSheetState extends State<CanvasCardSheet> {
  late final TextEditingController _text = TextEditingController(
    text: widget.card?.text ?? '',
  );

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  bool get _isText => widget.card == null || widget.card!.kind == 'text';

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final card = widget.card;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        SheetHeader(
          title: _isText ? l10n.canvasCardTitle : (card?.text ?? ''),
          actionLabel: l10n.canvasBack,
        ),
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              if (_isText) ...<Widget>[
                KansoTextField(
                  controller: _text,
                  hint: l10n.canvasCardHint,
                  minLines: 3,
                  maxLines: 8,
                  autofocus: true,
                  keyboardType: TextInputType.multiline,
                  autocorrect: true,
                ),
                const SizedBox(height: KansoSpacing.sm),
                KansoButton(
                  label: l10n.canvasCardSave,
                  onPressed: () =>
                      Navigator.of(context).pop(CardText(_text.text)),
                ),
              ] else ...<Widget>[
                if (card?.missing ?? false)
                  Padding(
                    padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
                    child: Text(
                      l10n.canvasCardMissing,
                      style: context.typography.secondary,
                    ),
                  )
                else
                  KansoButton(
                    label: l10n.canvasCardOpenNote,
                    icon: KansoIcons.fileText,
                    onPressed: () =>
                        Navigator.of(context).pop(const CardOpenNote()),
                  ),
              ],
              if (card != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                KansoButton(
                  label: l10n.canvasCardDelete,
                  icon: KansoIcons.trash,
                  variant: KansoButtonVariant.danger,
                  onPressed: () =>
                      Navigator.of(context).pop(const CardDelete()),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }
}
