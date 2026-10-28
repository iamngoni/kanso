// Kanso — daily_template_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/templates/state/daily_template/daily_template_bloc.dart';
import 'package:kanso/templates/state/template_picker/template_picker_bloc.dart';

import '../../helpers/fixture_journal_repository.dart';

void main() {
  late FixtureJournalRepository fixture;

  setUp(() => fixture = FixtureJournalRepository());

  group('DailyTemplateBloc', () {
    test('reads the templates and the one chosen', () async {
      fixture.daily = 'note:tpl-meeting';
      final bloc = DailyTemplateBloc(fixture)
        ..add(const DailyTemplateEvent.started());
      addTearDown(bloc.close);
      final state = await bloc.stream.firstWhere(
        (s) => s.status == LoadStatus.ready,
      );
      expect(state.templates, hasLength(3));
      expect(state.chosen?.title, 'Meeting notes');
    });

    test('a template that is gone is no template', () async {
      fixture.daily = 'note:gone';
      final bloc = DailyTemplateBloc(fixture)
        ..add(const DailyTemplateEvent.started());
      addTearDown(bloc.close);
      final state = await bloc.stream.firstWhere(
        (s) => s.status == LoadStatus.ready,
      );
      expect(state.current, isNull);
      expect(state.chosen, isNull);
    });

    test('choosing sets it, and None clears it', () async {
      final bloc = DailyTemplateBloc(fixture)
        ..add(const DailyTemplateEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(const DailyTemplateEvent.chosen('note:tpl-daily'));
      await bloc.stream.firstWhere((s) => s.current == 'note:tpl-daily');
      expect(fixture.daily, 'note:tpl-daily');
      bloc.add(const DailyTemplateEvent.chosen(null));
      await bloc.stream.firstWhere((s) => s.current == null);
      expect(fixture.daily, isNull);
      expect(fixture.calls, <String>['daily:note:tpl-daily', 'daily:none']);
    });

    test('a failure to set it keeps the old choice and says so', () async {
      fixture.daily = 'note:tpl-daily';
      final bloc = DailyTemplateBloc(fixture)
        ..add(const DailyTemplateEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      fixture.failure = const ApplicationError.storage(message: 'Disk.');
      bloc.add(const DailyTemplateEvent.chosen(null));
      final state = await bloc.stream.firstWhere((s) => s.error != null);
      expect(state.current, 'note:tpl-daily');
      bloc.add(const DailyTemplateEvent.errorDismissed());
      await bloc.stream.firstWhere((s) => s.error == null);
    });
  });

  group('TemplatePickerBloc', () {
    test('lists the templates', () async {
      final bloc = TemplatePickerBloc(fixture)
        ..add(const TemplatePickerEvent.started());
      addTearDown(bloc.close);
      final state = await bloc.stream.firstWhere(
        (s) => s.status == LoadStatus.ready,
      );
      expect(state.templates.map((t) => t.title), <String>[
        'Daily',
        'Meeting notes',
        '',
      ]);
    });

    test('a failure is a failure', () async {
      fixture.failure = const ApplicationError.storage(message: 'Disk.');
      final bloc = TemplatePickerBloc(fixture)
        ..add(const TemplatePickerEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.failed);
    });
  });
}
