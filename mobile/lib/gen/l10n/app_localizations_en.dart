// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'Kanso';

  @override
  String get libraryAllNotes => 'All Notes';

  @override
  String get libraryTemplates => 'Templates';

  @override
  String get libraryPinned => 'Pinned';

  @override
  String get libraryRecent => 'Recent';

  @override
  String get libraryNotebooks => 'Notebooks';

  @override
  String get libraryTags => 'Tags';

  @override
  String get libraryTrash => 'Trash';

  @override
  String get openSettings => 'Settings';

  @override
  String get openLibrary => 'Library';

  @override
  String get searchHint => 'Search notes';

  @override
  String get searchClear => 'Clear search';

  @override
  String get searchCancel => 'Cancel';

  @override
  String get searchFilterAll => 'All';

  @override
  String get searchFilterTitles => 'Titles';

  @override
  String get searchFilterTags => 'Tags';

  @override
  String get searchFilterNotebook => 'This notebook';

  @override
  String searchResultCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count results',
      one: '1 result',
    );
    return '$_temp0';
  }

  @override
  String noteListCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$_temp0';
  }

  @override
  String get noteListMore => 'More';

  @override
  String get noteBack => 'Back';

  @override
  String get noteForward => 'Forward';

  @override
  String get noteSearch => 'Search';

  @override
  String get notePreview => 'Preview';

  @override
  String get noteEdit => 'Edit';

  @override
  String get noteInfo => 'Note info';

  @override
  String get noteMetaCreated => 'Created';

  @override
  String get noteMetaUpdated => 'Updated';

  @override
  String get statusActive => 'Active';

  @override
  String get statusOnHold => 'On hold';

  @override
  String get statusCompleted => 'Completed';

  @override
  String get statusDropped => 'Dropped';

  @override
  String editProperties(int count) {
    return 'Properties · $count';
  }

  @override
  String get accessoryHeading => 'Heading';

  @override
  String get accessoryList => 'List';

  @override
  String get accessoryTask => 'Task';

  @override
  String get accessoryBold => 'Bold';

  @override
  String get accessoryLink => 'Link';

  @override
  String get accessoryImage => 'Image';

  @override
  String get accessoryWikiLink => 'Link to a note';

  @override
  String get accessoryIndent => 'Indent';

  @override
  String get accessoryOutdent => 'Outdent';

  @override
  String get accessoryRedo => 'Redo';

  @override
  String get accessoryUndo => 'Undo';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsClose => 'Close';

  @override
  String get settingsGeneral => 'General';

  @override
  String get settingsDefaultNotebook => 'Default notebook';

  @override
  String get defaultNotebookFirst => 'First notebook';

  @override
  String get defaultNotebookFooter =>
      'New notes go here, except those made from a notebook\'s own list. A locked notebook can\'t be chosen.';

  @override
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsTextSize => 'Text size';

  @override
  String get settingsSync => 'Sync';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsVersion => 'Version';

  @override
  String get settingsWebsite => 'Website';

  @override
  String get settingsReleaseNotes => 'Release notes';

  @override
  String get settingsHelp => 'Help';

  @override
  String get settingsLicences => 'Licences';

  @override
  String get themesTitle => 'Themes';

  @override
  String get themesInstalled => 'Installed';

  @override
  String get themesBadgeBuiltIn => 'Built in';

  @override
  String get themesBadgeInstalled => 'Installed';

  @override
  String get themesCurrent => 'In use';

  @override
  String get lockedTitle => 'This note is locked';

  @override
  String get errorSignedOutTitle => 'Signed out on this device';

  @override
  String get errorSignedOutMessage =>
      'Your session on this device ended. Sign in again to keep syncing; your notes stay here.';

  @override
  String get errorSignInAgain => 'Sign in again';

  @override
  String get errorNotNow => 'Not now';

  @override
  String get designGalleryTitle => 'Design proposal';

  @override
  String get designGalleryIntro =>
      'Static screens for review. Nothing here saves or syncs.';

  @override
  String get designGalleryPhone => 'Phone';

  @override
  String get designGalleryTablet => 'Tablet';

  @override
  String get appearanceSystem => 'System';

  @override
  String get appearanceLight => 'Light';

  @override
  String get appearanceDark => 'Dark';

  @override
  String get screenError => 'Error card';

  @override
  String get themesSampleHeading => 'Heading';

  @override
  String get themesSampleLine => 'Some **bold** and *italic*';

  @override
  String get themesSampleCode => '`code`, [a link](url)';

  @override
  String get themesSampleQuote => 'a quote';

  @override
  String get settingsFonts => 'Fonts';

  @override
  String get settingsBack => 'Back';

  @override
  String get fontsTitle => 'Fonts';

  @override
  String get fontsEditorSection => 'Editor font';

  @override
  String get fontsEditorFooter => 'The font you write in.';

  @override
  String get fontsPreviewSection => 'Preview font';

  @override
  String get fontsPreviewFooter => 'The font a note is read in.';

  @override
  String get fontsCodeSection => 'Code font';

  @override
  String get fontsCodeFooter => 'Used for code in notes and in theme previews.';

  @override
  String get fontsSystemName => 'System — your device’s font';

  @override
  String get fontsSystemShort => 'System';

  @override
  String get fontsSystemSample => 'San Francisco on iOS, Roboto on Android';

  @override
  String get fontsSampleText => 'Calm notes, clearly written';

  @override
  String get fontsSampleCode => 'let note = merge(a, b);';

  @override
  String get fontsTextSizeSection => 'Text size';

  @override
  String get fontsTextSizeFooter =>
      'Changes the text of notes only. The rest of Kanso follows your device’s text size.';

  @override
  String get fontsTextSizeSample =>
      'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.';

  @override
  String get textSizeSmall => 'Small';

  @override
  String get textSizeDefault => 'Default';

  @override
  String get textSizeLarge => 'Large';

  @override
  String get textSizeLarger => 'Larger';

  @override
  String get editorTitleHint => 'Title';

  @override
  String get editorChangedMessage =>
      'This note was changed elsewhere while you were editing. The version you don\'t keep stays in its history.';

  @override
  String get editorKeepMine => 'Keep mine';

  @override
  String get editorUseTheirs => 'Use theirs';

  @override
  String get editorDeletedMessage =>
      'This note was deleted elsewhere while you were editing.';

  @override
  String get editorKeepIt => 'Keep it';

  @override
  String get editorDiscard => 'Discard';

  @override
  String get editorSaveFailed => 'Your changes aren\'t saved yet.';

  @override
  String get editorOpeningLabel => 'Opening the note';

  @override
  String get propertiesAdd => 'Add property';

  @override
  String get propertiesName => 'Name';

  @override
  String get propertiesValue => 'Value';

  @override
  String propertiesRemove(String name) {
    return 'Remove $name';
  }

  @override
  String get propertiesConfirmAdd => 'Add';

  @override
  String get propertiesCancelAdd => 'Cancel';

  @override
  String get propertiesFailed => 'That property couldn\'t be set.';

  @override
  String get propertiesShow => 'Show properties';

  @override
  String get propertiesHide => 'Hide properties';

  @override
  String get noteModeLabel => 'View mode';

  @override
  String get noteListLoading => 'Loading notes';

  @override
  String emptyNotebookTitle(String notebook) {
    return 'No notes in $notebook yet';
  }

  @override
  String get emptyNotebookMessage =>
      'Notes added here will appear in this list.';

  @override
  String get noNoteSelectedTitle => 'No note selected';

  @override
  String get noNoteSelectedMessage =>
      'Choose a note from the list to read it here.';

  @override
  String get libraryDesignGallery => 'Design gallery';

  @override
  String get retry => 'Try again';

  @override
  String get libraryLoading => 'Loading the library';

  @override
  String get libraryFailedTitle => 'The library didn\'t load';

  @override
  String get libraryFailedMessage =>
      'Something went wrong reading the notebooks and tags on this device. Try again.';

  @override
  String get libraryNoNotebooks => 'No notebooks yet';

  @override
  String get libraryNoTags => 'No tags yet';

  @override
  String get dayToday => 'Today';

  @override
  String get dayYesterday => 'Yesterday';

  @override
  String get noteUntitled => 'Untitled';

  @override
  String get noteListLocked => 'Locked';

  @override
  String get noteListFailedTitle => 'These notes didn\'t load';

  @override
  String get noteListFailedMessage =>
      'Something went wrong reading the notes on this device. Try again.';

  @override
  String get emptyAllTitle => 'No notes yet';

  @override
  String get emptyAllMessage => 'Notes will appear here.';

  @override
  String get emptyPinnedTitle => 'Nothing pinned';

  @override
  String get emptyPinnedMessage =>
      'Pin a note by swiping it, and it will wait here.';

  @override
  String get emptyRecentTitle => 'Nothing recent';

  @override
  String get emptyRecentMessage =>
      'Notes you have changed lately will appear here.';

  @override
  String get emptyTrashTitle => 'The trash is empty';

  @override
  String get emptyTrashMessage =>
      'Notes you trash wait here until you move them out.';

  @override
  String emptyTagTitle(String tag) {
    return 'No notes tagged $tag';
  }

  @override
  String get emptyTagMessage => 'Notes with this tag will appear in this list.';

  @override
  String get emptyNarrowedTitle => 'No notes match';

  @override
  String get emptyNarrowedMessage =>
      'Try another status, or turn off Pinned Only.';

  @override
  String get listOptionsTitle => 'View options';

  @override
  String get listOptionsSortBy => 'Sort by';

  @override
  String get listOptionsDateUpdated => 'Date Updated';

  @override
  String get listOptionsDateCreated => 'Date Created';

  @override
  String get listOptionsTitleSort => 'Title';

  @override
  String get listOptionsNewestFirst => 'Newest First';

  @override
  String get listOptionsOldestFirst => 'Oldest First';

  @override
  String get listOptionsAToZ => 'A to Z';

  @override
  String get listOptionsZToA => 'Z to A';

  @override
  String get listOptionsStatus => 'Status';

  @override
  String get listOptionsAnyStatus => 'Any Status';

  @override
  String get listOptionsPinnedOnly => 'Pinned Only';

  @override
  String get noteListDone => 'Done';

  @override
  String noteListSelected(int count) {
    return '$count selected';
  }

  @override
  String get noteListPin => 'Pin';

  @override
  String get noteListUnpin => 'Unpin';

  @override
  String get noteListMove => 'Move';

  @override
  String get noteListTrash => 'Trash';

  @override
  String moveSheetTitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Move $count notes to',
      one: 'Move 1 note to',
    );
    return '$_temp0';
  }

  @override
  String get noteListCancel => 'Cancel';

  @override
  String get noteListChangeFailed => 'That didn\'t work. Try again.';

  @override
  String get noteListChangeLocked =>
      'Locked notebooks can\'t take or give up notes here yet.';

  @override
  String get libraryNewNotebook => 'New notebook';

  @override
  String get newNotebookTitle => 'New notebook';

  @override
  String get newNotebookNameHint => 'Name';

  @override
  String get newNotebookCreate => 'Create';

  @override
  String get newNotebookInvalid => 'Give the notebook a name.';

  @override
  String get newNotebookFailed => 'The notebook couldn\'t be made. Try again.';

  @override
  String get noteListNewNote => 'New note';

  @override
  String get noteListNewNoteLocked =>
      'Unlock this notebook to add notes to it.';

  @override
  String get noteListNewNoteFailed => 'The note couldn\'t be made. Try again.';

  @override
  String get noteListDismiss => 'OK';

  @override
  String get noteLoading => 'Loading the note';

  @override
  String get noteNotFoundTitle => 'This note isn\'t here';

  @override
  String get noteNotFoundMessage =>
      'It may have been deleted, or it hasn\'t reached this device yet.';

  @override
  String get noteFailedTitle => 'This note didn\'t load';

  @override
  String get noteFailedMessage =>
      'Something went wrong reading the note on this device. Try again.';

  @override
  String get lockedMessageSealed =>
      'Its text stays sealed. Only its title can be shown here.';

  @override
  String get menuCut => 'Cut';

  @override
  String get menuCopy => 'Copy';

  @override
  String get menuPaste => 'Paste';

  @override
  String get menuSelectAll => 'Select All';

  @override
  String get menuDelete => 'Delete';

  @override
  String get menuOpenLink => 'Open link';

  @override
  String completionAnnouncement(int count, String label) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count suggestions',
      one: '1 suggestion',
    );
    return '$_temp0, $label selected';
  }

  @override
  String get searchIdleTitle => 'Search your notes';

  @override
  String get searchIdleMessage => 'Find words in titles and text, or a tag.';

  @override
  String get searchNoResultsTitle => 'No notes match';

  @override
  String searchNoResultsMessage(String query) {
    return 'Nothing matched “$query”. Try fewer or different words.';
  }

  @override
  String get searchFailedTitle => 'The search didn\'t run';

  @override
  String get searchFailedMessage =>
      'Something went wrong searching the notes on this device. Try again.';

  @override
  String get searchLoading => 'Searching';

  @override
  String settingsThemeBoth(String light, String dark) {
    return '$light, $dark';
  }

  @override
  String get settingsSyncStatus => 'Status';

  @override
  String get settingsSyncOff => 'Off';

  @override
  String get settingsSyncOn => 'On';

  @override
  String get settingsSyncSyncing => 'Syncing…';

  @override
  String get settingsSyncSynced => 'Synced';

  @override
  String get settingsSyncAttention => 'Needs attention';

  @override
  String get settingsSyncBackupFooter =>
      'The notes on this device aren’t included in device backups.';

  @override
  String settingsVersionValue(String version, String build) {
    return '$version ($build)';
  }

  @override
  String get licencesLoading => 'Loading licences';

  @override
  String get licencesFailedTitle => 'The licences didn’t load';

  @override
  String get licencesFailedMessage =>
      'Something went wrong reading the licences. Try again.';

  @override
  String get licencesEmptyTitle => 'No licences';

  @override
  String get licencesEmptyMessage =>
      'The packages Kanso is made of will be listed here.';

  @override
  String licencesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count licences',
      one: '1 licence',
    );
    return '$_temp0';
  }

  @override
  String get themesFollowSystem =>
      'Switch between a light and a dark theme with the system';

  @override
  String get themesFollowSystemOn =>
      'Showing the light theme in light mode and the dark one in dark mode.';

  @override
  String get themesFollowSystemOff =>
      'Off: the theme stays as picked whatever the system’s mode.';

  @override
  String get themesFailedTitle => 'The themes didn’t load';

  @override
  String get themesFailedMessage =>
      'Something went wrong reading the themes. Try again.';

  @override
  String get themesLoading => 'Loading themes';

  @override
  String get backupNoticeTitle => 'Notes stay on this device';

  @override
  String get backupNoticeMessage =>
      'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.';

  @override
  String get backupNoticeOk => 'OK';

  @override
  String settingsVersionPatchValue(String version, String build, int patch) {
    return '$version ($build), patch $patch';
  }

  @override
  String get settingsPrivacy => 'Privacy';

  @override
  String get settingsPrivacyFooter =>
      'Kanso sends a crash report only when you choose to.';

  @override
  String get crashReportsTitle => 'Crash reports';

  @override
  String get crashReportsLoading => 'Loading crash reports';

  @override
  String get crashReportsEmptyTitle => 'No crash reports';

  @override
  String get crashReportsEmptyMessage =>
      'If Kanso closes unexpectedly, a report waits here until you decide whether to send it.';

  @override
  String get crashReportsIntro =>
      'A report holds only where in Kanso’s code something went wrong. It never holds note text, titles or file names. Nothing is sent unless you send it.';

  @override
  String get crashReportsSendAll => 'Send all';

  @override
  String crashReportSubtitle(String day, String kind) {
    return '$day · $kind';
  }

  @override
  String get crashReportSend => 'Send';

  @override
  String get crashReportDelete => 'Delete';

  @override
  String get crashReportShow => 'Show report';

  @override
  String get crashReportHide => 'Hide report';

  @override
  String get crashReportSending => 'Sending…';

  @override
  String get crashKindDart => 'App';

  @override
  String get crashKindIosNative => 'iOS';

  @override
  String get crashKindAndroidNative => 'Android native';

  @override
  String get crashKindAndroidAnr => 'Android not responding';

  @override
  String get crashKindAndroidJvm => 'Android';

  @override
  String get crashNoticeTitle => 'Kanso closed unexpectedly';

  @override
  String get crashNoticeMessage =>
      'Send a crash report? It holds only where in Kanso’s code it happened, never note text, titles or file names.';

  @override
  String get crashNoticeSend => 'Send';

  @override
  String get crashNoticeDiscard => 'Don’t send';

  @override
  String get crashOutcomeSent => 'Report sent. Thank you.';

  @override
  String get crashOutcomeRejected => 'The report couldn’t be sent.';

  @override
  String get crashOutcomeTryLater =>
      'Too many reports just now. Try again later.';

  @override
  String get crashOutcomeUnreachable =>
      'Couldn’t reach Kanso. Try again later.';

  @override
  String get crashReportsFailedMessage => 'The crash reports couldn’t be read.';
}
