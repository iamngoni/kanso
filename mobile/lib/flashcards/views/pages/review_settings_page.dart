// Kanso — review_settings_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/cards.dart';
import '../../../core/bridge/api/cards_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../utils/flashcards_text.dart';

/// Settings › Flashcards, at `/review/settings`: the daily limits, the
/// target recall, the daily reminder, the editor's card icons and a reset of
/// the review history.
class ReviewSettingsPage extends StatefulWidget {
  const ReviewSettingsPage({super.key});

  @override
  State<ReviewSettingsPage> createState() => _ReviewSettingsPageState();
}

class _ReviewSettingsPageState extends State<ReviewSettingsPage> {
  late final KansoCards _cards = getIt<KansoLibrary>().cards();
  CardSettingsView? _settings;
  String? _error;

  @override
  void initState() {
    super.initState();
    _cards
        .settings()
        .then((s) {
          if (mounted) {
            setState(() => _settings = s);
          }
        })
        .catchError((Object e) {
          if (mounted) {
            setState(() => _error = applicationErrorOf(e).message);
          }
        });
  }

  Future<void> _save(CardSettingsView next) async {
    try {
      final saved = await _cards.setSettings(input: next);
      if (mounted) {
        setState(() {
          _settings = saved;
          _error = null;
        });
      }
    } on Object catch (e) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(e).message);
      }
    }
  }

  CardSettingsView _with(
    CardSettingsView s, {
    int? newPerDay,
    int? reviewsPerDay,
    double? retention,
    String? reminder,
    bool clearReminder = false,
    bool? showIcons,
  }) => CardSettingsView(
    newPerDay: newPerDay ?? s.newPerDay,
    reviewsPerDay: reviewsPerDay ?? s.reviewsPerDay,
    retention: retention ?? s.retention,
    reminder: clearReminder ? null : (reminder ?? s.reminder),
    showIcons: showIcons ?? s.showIcons,
  );

  Future<void> _pickTime(CardSettingsView s) async {
    final now = TimeOfDay.now();
    final picked = await showTimePicker(
      context: context,
      initialTime: now,
    );
    if (picked != null) {
      final hh = picked.hour.toString().padLeft(2, '0');
      final mm = picked.minute.toString().padLeft(2, '0');
      final text = '$hh:$mm';
      await _save(_with(s, reminder: text));
    }
  }

  Future<void> _reset() async {
    final yes = await KansoConfirmSheet.show(
      context,
      title: FlashcardsText.resetTitle,
      message: FlashcardsText.resetBody,
      confirmLabel: FlashcardsText.reset,
    );
    if (yes) {
      try {
        await _cards.resetHistory();
      } on Object catch (e) {
        if (mounted) {
          setState(() => _error = applicationErrorOf(e).message);
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final s = _settings;
    return KansoPage(
      backgroundColor: colors.app,
      body: ColoredBox(
        color: colors.app,
        child: Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              bordered: false,
              leading: const ShellBackButton(),
              title: FlashcardsText.settings,
            ),
            Expanded(
              child: s == null
                  ? Center(
                      child: _error == null
                          ? const CircularProgressIndicator()
                          : Text(_error!),
                    )
                  : ListView(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      children: <Widget>[
                        _Stepper(
                          label: FlashcardsText.newPerDay,
                          value: s.newPerDay,
                          step: 5,
                          onChanged: (v) => _save(_with(s, newPerDay: v)),
                        ),
                        _Stepper(
                          label: FlashcardsText.reviewsPerDay,
                          value: s.reviewsPerDay,
                          step: 25,
                          onChanged: (v) => _save(_with(s, reviewsPerDay: v)),
                        ),
                        const SizedBox(height: KansoSpacing.md),
                        Text(
                          '${FlashcardsText.targetRecall}: '
                          '${(s.retention * 100).round()}%',
                          style: type.body,
                        ),
                        Slider(
                          min: 0.80,
                          max: 0.97,
                          divisions: 17,
                          value: s.retention.clamp(0.80, 0.97),
                          onChanged: (v) => setState(
                            () => _settings = _with(s, retention: v),
                          ),
                          onChangeEnd: (v) => _save(_with(s, retention: v)),
                        ),
                        ListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            FlashcardsText.dailyReminder,
                            style: type.body,
                          ),
                          subtitle: Text(
                            s.reminder ?? FlashcardsText.reminderOff,
                            style: type.meta.copyWith(
                              color: colors.textSecondary,
                            ),
                          ),
                          trailing: s.reminder == null
                              ? null
                              : TextButton(
                                  onPressed: () => _save(
                                    _with(s, clearReminder: true),
                                  ),
                                  child: const Text(FlashcardsText.reminderOff),
                                ),
                          onTap: () => _pickTime(s),
                        ),
                        SwitchListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            FlashcardsText.showIcons,
                            style: type.body,
                          ),
                          value: s.showIcons,
                          onChanged: (v) => _save(_with(s, showIcons: v)),
                        ),
                        if (_error case final message?)
                          Text(
                            message,
                            style: type.meta.copyWith(color: colors.danger),
                          ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoButton(
                          label: FlashcardsText.resetHistory,
                          variant: KansoButtonVariant.danger,
                          onPressed: _reset,
                        ),
                      ],
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

class _Stepper extends StatelessWidget {
  const _Stepper({
    required this.label,
    required this.value,
    required this.step,
    required this.onChanged,
  });

  final String label;
  final int value;
  final int step;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        Expanded(child: Text(label, style: context.typography.body)),
        IconButton(
          icon: const Icon(Icons.remove),
          onPressed: value <= 0
              ? null
              : () => onChanged((value - step).clamp(0, 10000)),
        ),
        SizedBox(
          width: 44,
          child: Text(
            '$value',
            textAlign: TextAlign.center,
            style: context.typography.label,
          ),
        ),
        IconButton(
          icon: const Icon(Icons.add),
          onPressed: () => onChanged((value + step).clamp(0, 10000)),
        ),
      ],
    );
  }
}
