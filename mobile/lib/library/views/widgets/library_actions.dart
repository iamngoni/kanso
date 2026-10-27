// Kanso — library_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter/widgets.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import 'library_message_sheet.dart';

/// Does [action] to the library, then has the index read what changed; when
/// it fails, says so over [host].
///
/// [host] must outlive the sheet the action was chosen from: it is the
/// root navigator's context, since the sheet is popped before this runs.
/// [index] is the bloc that keeps the library's notebooks and tags, which
/// the sheet was given because it sits above the pages' providers.
Future<bool> runLibraryAction(
  BuildContext host, {
  required LibraryIndexBloc index,
  required Future<Either<ApplicationError, Object?>> Function() action,
  List<String> changed = const <String>[],
}) async {
  final result = await action();
  return result.fold(
    (error) {
      if (host.mounted) {
        LibraryMessageSheet.show(host, host.l10n.libraryActionFailed);
      }
      return false;
    },
    (_) {
      index.add(LibraryIndexEvent.changesReceived(changed));
      return true;
    },
  );
}
