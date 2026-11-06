// Kanso — people_list_page.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/ai_trouble_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/repos/abs/people_repository.dart';
import '../widgets/people_name_sheet.dart';

/// People & Projects, at `/people`: the names Kanso suggests pages for, and
/// the pages there are. Nothing is made until the person says so.
class PeopleListPage extends StatefulWidget {
  const PeopleListPage({super.key});

  @override
  State<PeopleListPage> createState() => _PeopleListPageState();
}

class _PeopleListPageState extends State<PeopleListPage> {
  final PeopleRepository _people = getIt<PeopleRepository>();
  List<PeoplePage>? _pages;
  List<PeopleSuggestion> _suggestions = const <PeopleSuggestion>[];
  PeopleStatus? _status;
  String? _failure;
  bool _labelling = false;
  String? _aiFailure;

  @override
  void initState() {
    super.initState();
    unawaited(_load(detect: true));
  }

  Future<void> _load({bool detect = false}) async {
    if (detect) {
      // The core decides whether a look is due.
      await _people.detect(force: false);
    }
    final pages = await _people.pages();
    final suggestions = await _people.suggestions();
    final status = await _people.status();
    if (!mounted) {
      return;
    }
    setState(() {
      _pages = pages.toOption().toNullable() ?? _pages ?? <PeoplePage>[];
      _suggestions =
          suggestions.toOption().toNullable() ?? const <PeopleSuggestion>[];
      _status = status.toOption().toNullable();
      _failure = pages.fold((e) => e.message, (_) => null);
    });
  }

  Future<void> _made(String name, PeopleKind kind, List<String> also) async {
    final made = await _people.create(kind, name, also);
    if (!mounted) {
      return;
    }
    final id = made.fold((e) {
      setState(() => _failure = e.message);
      return null;
    }, (id) => id);
    await _load();
    if (id != null && mounted) {
      unawaited(context.push(citedNoteLocation(id)));
    }
  }

  Future<void> _new(PeopleKind kind) async {
    final answer = await PeopleNameSheet.show(context, kind);
    if (answer != null) {
      await _made(answer.name, kind, answer.aliases);
    }
  }

  Future<void> _dismiss(PeopleSuggestion s, {required bool never}) async {
    await _people.dismiss(s.key, never: never);
    await _load();
  }

  Future<void> _sameAs(PeopleSuggestion s, {required bool yes}) async {
    if (yes) {
      await _people.confirmAlias(s.key, s.aliasOfId);
    } else {
      await _people.dismiss(s.key, never: false);
    }
    await _load();
  }

  Future<void> _label() async {
    setState(() {
      _labelling = true;
      _aiFailure = null;
    });
    final done = await _people.detectWithAi();
    if (!mounted) {
      return;
    }
    setState(() {
      _labelling = false;
      _aiFailure = done.fold((e) => e.message, (_) => null);
    });
    await _load();
  }

  Future<void> _newMenu(BuildContext button) async {
    final box = button.findRenderObject() as RenderBox?;
    if (box == null) {
      return;
    }
    final l10n = context.l10n;
    final at = box.localToGlobal(Offset(0, box.size.height));
    final chosen = await showKansoMenu<PeopleKind>(
      button,
      at,
      <KansoMenuEntry<PeopleKind>>[
        KansoMenuEntry<PeopleKind>(
          PeopleKind.person,
          l10n.peopleNewPerson,
          icon: KansoIcons.users,
        ),
        KansoMenuEntry<PeopleKind>(
          PeopleKind.project,
          l10n.peopleNewProject,
          icon: KansoIcons.folder,
        ),
      ],
    );
    if (chosen != null) {
      unawaited(_new(chosen));
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final typography = context.typography;
    final pages = _pages;
    final status = _status;
    final failure = _failure;
    final aiFailure = _aiFailure;
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
              title: l10n.peopleTitle,
              subtitle: pages == null
                  ? null
                  : l10n.peopleSubtitle(pages.length),
              trailing: <Widget>[
                Builder(
                  builder: (button) => KansoIconButton(
                    icon: KansoIcons.plus,
                    semanticLabel: l10n.peopleCreate,
                    onPressed: () => unawaited(_newMenu(button)),
                  ),
                ),
                KansoIconButton(
                  icon: KansoIcons.settings,
                  semanticLabel: l10n.peopleSettingsTitle,
                  onPressed: () => context.push(KansoRoutes.settingsPeople),
                ),
              ],
            ),
            if (pages != null)
              Expanded(
                child: pages.isEmpty && _suggestions.isEmpty
                    ? KansoEmptyState(
                        title: l10n.peopleEmptyTitle,
                        message: status != null && !status.suggestOn
                            ? l10n.peopleSuggestionsOff
                            : l10n.peopleEmptyMessage,
                        action: KansoButton(
                          label: l10n.peopleNewPerson,
                          expanded: false,
                          onPressed: () => unawaited(_new(PeopleKind.person)),
                        ),
                      )
                    : KansoListScrollbar(
                        child: ListView(
                          padding: const EdgeInsets.all(KansoSpacing.md),
                          children: <Widget>[
                            if (failure != null)
                              KansoInlineMessage(message: failure),
                            if (_suggestions.isNotEmpty) ...<Widget>[
                              KansoSectionHeader(
                                label: l10n.peopleSuggestionsTitle,
                                horizontalPadding: 0,
                              ),
                              for (final s in _suggestions)
                                _suggestion(context, s),
                              if (status != null && status.aiOn)
                                Align(
                                  alignment: AlignmentDirectional.centerStart,
                                  child: KansoButton(
                                    label: _labelling
                                        ? '…'
                                        : l10n.peopleSettingsDetectAi,
                                    variant: KansoButtonVariant.quiet,
                                    compact: true,
                                    expanded: false,
                                    onPressed: _labelling
                                        ? null
                                        : () => unawaited(_label()),
                                  ),
                                ),
                              if (aiFailure != null)
                                Text(
                                  aiTroubleText(aiFailure, l10n),
                                  style: typography.secondary.copyWith(
                                    color: colors.danger,
                                  ),
                                ),
                              const SizedBox(height: KansoSpacing.lg),
                            ],
                            if (pages.isNotEmpty) ...<Widget>[
                              KansoSectionHeader(
                                label: l10n.peoplePagesTitle,
                                horizontalPadding: 0,
                              ),
                              KansoGroupedSection(
                                dividerIndent: 48,
                                children: <Widget>[
                                  for (final p in pages)
                                    KansoSettingsRow(
                                      icon: p.kind == PeopleKind.person
                                          ? KansoIcons.users
                                          : KansoIcons.folder,
                                      label: p.title,
                                      value: p.detail,
                                      chevron: true,
                                      onTap: () =>
                                          context.push(citedNoteLocation(p.id)),
                                    ),
                                ],
                              ),
                            ],
                          ],
                        ),
                      ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _suggestion(BuildContext context, PeopleSuggestion s) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final person = s.kind == PeopleKind.person;
    final alias = s.aliasOfId.isNotEmpty;
    return Padding(
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      child: Container(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        decoration: BoxDecoration(
          color: colors.elevated,
          border: Border.all(color: colors.divider),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Text(s.name, style: typography.label),
            Text(
              s.detail,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
            if (s.alsoLine.isNotEmpty)
              Text(
                s.alsoLine,
                style: typography.meta.copyWith(color: colors.textMuted),
              ),
            const SizedBox(height: KansoSpacing.xs),
            if (alias) ...<Widget>[
              Text(
                l10n.peopleAliasQuestion(s.name, s.aliasOfTitle),
                style: typography.secondary,
              ),
              const SizedBox(height: KansoSpacing.xs),
              Wrap(
                spacing: KansoSpacing.xs,
                runSpacing: KansoSpacing.xxs,
                children: <Widget>[
                  KansoButton(
                    label: l10n.peopleAliasYes,
                    compact: true,
                    expanded: false,
                    onPressed: () => unawaited(_sameAs(s, yes: true)),
                  ),
                  KansoButton(
                    label: l10n.peopleAliasNo,
                    variant: KansoButtonVariant.secondary,
                    compact: true,
                    expanded: false,
                    onPressed: () => unawaited(_sameAs(s, yes: false)),
                  ),
                ],
              ),
            ] else
              Wrap(
                spacing: KansoSpacing.xs,
                runSpacing: KansoSpacing.xxs,
                children: <Widget>[
                  KansoButton(
                    label: l10n.peopleCreatePage,
                    compact: true,
                    expanded: false,
                    onPressed: () =>
                        unawaited(_made(s.name, s.kind, const <String>[])),
                  ),
                  KansoButton(
                    label: person
                        ? l10n.peopleNotPerson
                        : l10n.peopleNotProject,
                    variant: KansoButtonVariant.secondary,
                    compact: true,
                    expanded: false,
                    onPressed: () => unawaited(_dismiss(s, never: true)),
                  ),
                  KansoButton(
                    label: l10n.peopleNotNow,
                    variant: KansoButtonVariant.quiet,
                    compact: true,
                    expanded: false,
                    onPressed: () => unawaited(_dismiss(s, never: false)),
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }
}
