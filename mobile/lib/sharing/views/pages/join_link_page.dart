// Kanso — join_link_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/join_link/join_link_bloc.dart';
import '../widgets/join_link_pane.dart';

/// Join with a link: asks to join the note an invite link is for.
class JoinLinkPage extends StatelessWidget {
  const JoinLinkPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<JoinLinkBloc>(
      create: (context) => getIt<JoinLinkBloc>(),
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.app,
        body: const JoinLinkPane(),
      ),
    );
  }
}
