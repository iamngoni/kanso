// Kanso — parent_only_home.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../../injection.dart';
import '../../models/repos/abs/notebook_home_repository.dart';
import 'notebook_home_card.dart';

/// What a notebook with no notes of its own shows: if it has notebooks
/// inside it and homepages are on, its homepage (a page that takes this
/// one's place on a phone, the note pane on a wide window); otherwise
/// [empty], the usual empty list.
class ParentOnlyHome extends StatefulWidget {
  const ParentOnlyHome({
    required this.bookId,
    required this.empty,
    this.beside = false,
    super.key,
  });

  final String bookId;

  /// The empty list, for a notebook that has no homepage to open.
  final Widget empty;

  /// Whether the list is beside the note pane.
  final bool beside;

  @override
  State<ParentOnlyHome> createState() => _ParentOnlyHomeState();
}

class _ParentOnlyHomeState extends State<ParentOnlyHome> {
  /// Null until the card is read; then whether the homepage opens instead.
  bool? _opens;

  @override
  void initState() {
    super.initState();
    unawaited(_decide());
  }

  @override
  void didUpdateWidget(ParentOnlyHome old) {
    super.didUpdateWidget(old);
    if (old.bookId != widget.bookId) {
      _opens = null;
      unawaited(_decide());
    }
  }

  Future<void> _decide() async {
    final id = widget.bookId;
    final card = (await getIt<NotebookHomeRepository>().card(
      id,
    )).toOption().toNullable();
    if (!mounted || id != widget.bookId) {
      return;
    }
    final opens = card != null && card.show_ && card.opensDirectly;
    setState(() => _opens = opens);
    if (opens) {
      openNotebookHome(context, id, beside: widget.beside, replace: true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return switch (_opens) {
      false => widget.empty,
      _ => const SizedBox.shrink(),
    };
  }
}
