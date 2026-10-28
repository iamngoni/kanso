// Kanso — shared_with_you_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../widgets/shared_with_you_pane.dart';

/// Shared with you: the invitations to notes other people shared. It reads
/// the app's `InvitationsBloc`, which the library's row for it asks to read
/// again as it opens this.
class SharedWithYouPage extends StatelessWidget {
  const SharedWithYouPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.app,
      body: const SharedWithYouPane(),
    );
  }
}
