import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en')];

  /// The app's name, shown as its title.
  ///
  /// In en, this message translates to:
  /// **'Kanso'**
  String get appName;

  /// The library row that lists every note.
  ///
  /// In en, this message translates to:
  /// **'All Notes'**
  String get libraryAllNotes;

  /// The library row for note templates.
  ///
  /// In en, this message translates to:
  /// **'Templates'**
  String get libraryTemplates;

  /// The library row for pinned notes, and the list's group of them.
  ///
  /// In en, this message translates to:
  /// **'Pinned'**
  String get libraryPinned;

  /// The library row for recently opened notes.
  ///
  /// In en, this message translates to:
  /// **'Recent'**
  String get libraryRecent;

  /// The label above the list of notebooks.
  ///
  /// In en, this message translates to:
  /// **'Notebooks'**
  String get libraryNotebooks;

  /// The label above the list of tags.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get libraryTags;

  /// The library row for deleted notes.
  ///
  /// In en, this message translates to:
  /// **'Trash'**
  String get libraryTrash;

  /// What the settings button does.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get openSettings;

  /// What the button that opens the library does.
  ///
  /// In en, this message translates to:
  /// **'Library'**
  String get openLibrary;

  /// The hint in an empty search field.
  ///
  /// In en, this message translates to:
  /// **'Search notes'**
  String get searchHint;

  /// What the clear button of a search field does.
  ///
  /// In en, this message translates to:
  /// **'Clear search'**
  String get searchClear;

  /// Leaves the search.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get searchCancel;

  /// A search filter: everything.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get searchFilterAll;

  /// A search filter: only note titles.
  ///
  /// In en, this message translates to:
  /// **'Titles'**
  String get searchFilterTitles;

  /// A search filter: only tags.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get searchFilterTags;

  /// A search filter: the open notebook only.
  ///
  /// In en, this message translates to:
  /// **'This notebook'**
  String get searchFilterNotebook;

  /// How many notes match a search.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 result} other{{count} results}}'**
  String searchResultCount(int count);

  /// How many notes a list holds.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 note} other{{count} notes}}'**
  String noteListCount(int count);

  /// What the more button of the note list does.
  ///
  /// In en, this message translates to:
  /// **'More'**
  String get noteListMore;

  /// Goes back.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get noteBack;

  /// Goes forward.
  ///
  /// In en, this message translates to:
  /// **'Forward'**
  String get noteForward;

  /// Opens search.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get noteSearch;

  /// Shows the note rendered.
  ///
  /// In en, this message translates to:
  /// **'Preview'**
  String get notePreview;

  /// Shows the note as text to edit.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get noteEdit;

  /// Shows the note's details.
  ///
  /// In en, this message translates to:
  /// **'Note info'**
  String get noteInfo;

  /// The label of when a note was made.
  ///
  /// In en, this message translates to:
  /// **'Created'**
  String get noteMetaCreated;

  /// The label of when a note last changed.
  ///
  /// In en, this message translates to:
  /// **'Updated'**
  String get noteMetaUpdated;

  /// A note's status: in progress.
  ///
  /// In en, this message translates to:
  /// **'Active'**
  String get statusActive;

  /// A note's status: paused.
  ///
  /// In en, this message translates to:
  /// **'On hold'**
  String get statusOnHold;

  /// A note's status: done.
  ///
  /// In en, this message translates to:
  /// **'Completed'**
  String get statusCompleted;

  /// A note's status: given up.
  ///
  /// In en, this message translates to:
  /// **'Dropped'**
  String get statusDropped;

  /// The collapsed row of a note's properties, with how many it has.
  ///
  /// In en, this message translates to:
  /// **'Properties · {count}'**
  String editProperties(int count);

  /// A keyboard shortcut that starts a heading.
  ///
  /// In en, this message translates to:
  /// **'Heading'**
  String get accessoryHeading;

  /// A keyboard shortcut that starts a list.
  ///
  /// In en, this message translates to:
  /// **'List'**
  String get accessoryList;

  /// A keyboard shortcut that starts a task.
  ///
  /// In en, this message translates to:
  /// **'Task'**
  String get accessoryTask;

  /// A keyboard shortcut that makes text bold.
  ///
  /// In en, this message translates to:
  /// **'Bold'**
  String get accessoryBold;

  /// A keyboard shortcut that makes a link.
  ///
  /// In en, this message translates to:
  /// **'Link'**
  String get accessoryLink;

  /// A keyboard shortcut that adds an image.
  ///
  /// In en, this message translates to:
  /// **'Image'**
  String get accessoryImage;

  /// A keyboard shortcut that starts a link to another note.
  ///
  /// In en, this message translates to:
  /// **'Link to a note'**
  String get accessoryWikiLink;

  /// A keyboard shortcut that indents a line.
  ///
  /// In en, this message translates to:
  /// **'Indent'**
  String get accessoryIndent;

  /// A keyboard shortcut that outdents a line.
  ///
  /// In en, this message translates to:
  /// **'Outdent'**
  String get accessoryOutdent;

  /// What the undo key of the keyboard bar does when it is held, for a screen reader's list of actions.
  ///
  /// In en, this message translates to:
  /// **'Redo'**
  String get accessoryRedo;

  /// A keyboard shortcut that undoes the last change.
  ///
  /// In en, this message translates to:
  /// **'Undo'**
  String get accessoryUndo;

  /// The title of the settings page.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// Closes a page.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get settingsClose;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get settingsGeneral;

  /// The notebook new notes go to.
  ///
  /// In en, this message translates to:
  /// **'Default notebook'**
  String get settingsDefaultNotebook;

  /// The choice of the first notebook in the library as the one new notes go to.
  ///
  /// In en, this message translates to:
  /// **'First notebook'**
  String get defaultNotebookFirst;

  /// The note under the notebooks that can be the default one.
  ///
  /// In en, this message translates to:
  /// **'New notes go here, except those made from a notebook\'s own list. A locked notebook can\'t be chosen.'**
  String get defaultNotebookFooter;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get settingsAppearance;

  /// The colour theme.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get settingsTheme;

  /// The size of text in notes.
  ///
  /// In en, this message translates to:
  /// **'Text size'**
  String get settingsTextSize;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Sync'**
  String get settingsSync;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get settingsAbout;

  /// The app's version.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get settingsVersion;

  /// Opens Kanso's website.
  ///
  /// In en, this message translates to:
  /// **'Website'**
  String get settingsWebsite;

  /// Opens what changed in each version of Kanso, on the website.
  ///
  /// In en, this message translates to:
  /// **'Release notes'**
  String get settingsReleaseNotes;

  /// Opens Kanso's help page on the website.
  ///
  /// In en, this message translates to:
  /// **'Help'**
  String get settingsHelp;

  /// Opens the licences of what Kanso is made of.
  ///
  /// In en, this message translates to:
  /// **'Licences'**
  String get settingsLicences;

  /// The title of the theme page.
  ///
  /// In en, this message translates to:
  /// **'Themes'**
  String get themesTitle;

  /// The label above the installed themes.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get themesInstalled;

  /// A badge on a theme that comes with Kanso.
  ///
  /// In en, this message translates to:
  /// **'Built in'**
  String get themesBadgeBuiltIn;

  /// A badge on a theme that is installed.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get themesBadgeInstalled;

  /// Marks the theme in use.
  ///
  /// In en, this message translates to:
  /// **'In use'**
  String get themesCurrent;

  /// The title of the card shown in place of a locked note.
  ///
  /// In en, this message translates to:
  /// **'This note is locked'**
  String get lockedTitle;

  /// The title of the card shown when the session ended.
  ///
  /// In en, this message translates to:
  /// **'Signed out on this device'**
  String get errorSignedOutTitle;

  /// Says what happened and that no notes are lost.
  ///
  /// In en, this message translates to:
  /// **'Your session on this device ended. Sign in again to keep syncing; your notes stay here.'**
  String get errorSignedOutMessage;

  /// Starts signing in again.
  ///
  /// In en, this message translates to:
  /// **'Sign in again'**
  String get errorSignInAgain;

  /// Dismisses the card.
  ///
  /// In en, this message translates to:
  /// **'Not now'**
  String get errorNotNow;

  /// The title of the page that lists the mock screens.
  ///
  /// In en, this message translates to:
  /// **'Design proposal'**
  String get designGalleryTitle;

  /// Explains the page that lists the mock screens.
  ///
  /// In en, this message translates to:
  /// **'Static screens for review. Nothing here saves or syncs.'**
  String get designGalleryIntro;

  /// The group of mock screens drawn for a phone.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get designGalleryPhone;

  /// The group of mock screens drawn for a tablet.
  ///
  /// In en, this message translates to:
  /// **'Tablet'**
  String get designGalleryTablet;

  /// Follow the device's light or dark setting.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get appearanceSystem;

  /// Always light.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get appearanceLight;

  /// Always dark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get appearanceDark;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Error card'**
  String get screenError;

  /// The heading in a theme's sample Markdown; the # mark is added.
  ///
  /// In en, this message translates to:
  /// **'Heading'**
  String get themesSampleHeading;

  /// A line of a theme's sample Markdown with bold and italic text.
  ///
  /// In en, this message translates to:
  /// **'Some **bold** and *italic*'**
  String get themesSampleLine;

  /// A line of a theme's sample Markdown with code and a link.
  ///
  /// In en, this message translates to:
  /// **'`code`, [a link](url)'**
  String get themesSampleCode;

  /// The quote in a theme's sample Markdown; the > mark is added.
  ///
  /// In en, this message translates to:
  /// **'a quote'**
  String get themesSampleQuote;

  /// The settings row that opens the fonts of notes.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get settingsFonts;

  /// Goes back to the page before.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get settingsBack;

  /// The title of the page where the fonts of notes are chosen.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get fontsTitle;

  /// The section of fonts to write in.
  ///
  /// In en, this message translates to:
  /// **'Editor font'**
  String get fontsEditorSection;

  /// Explains the editor font.
  ///
  /// In en, this message translates to:
  /// **'The font you write in.'**
  String get fontsEditorFooter;

  /// The section of fonts to read notes in.
  ///
  /// In en, this message translates to:
  /// **'Preview font'**
  String get fontsPreviewSection;

  /// Explains the preview font.
  ///
  /// In en, this message translates to:
  /// **'The font a note is read in.'**
  String get fontsPreviewFooter;

  /// The section of fonts for code.
  ///
  /// In en, this message translates to:
  /// **'Code font'**
  String get fontsCodeSection;

  /// Explains the code font.
  ///
  /// In en, this message translates to:
  /// **'Used for code in notes and in theme previews.'**
  String get fontsCodeFooter;

  /// The choice that uses the font of the device, not one of Kanso’s.
  ///
  /// In en, this message translates to:
  /// **'System — your device’s font'**
  String get fontsSystemName;

  /// The short name of the device’s own font, for a summary.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get fontsSystemShort;

  /// Says which font the device’s own font is on each platform.
  ///
  /// In en, this message translates to:
  /// **'San Francisco on iOS, Roboto on Android'**
  String get fontsSystemSample;

  /// A short line shown in each font so it can be judged.
  ///
  /// In en, this message translates to:
  /// **'Calm notes, clearly written'**
  String get fontsSampleText;

  /// A short line of code shown in each code font.
  ///
  /// In en, this message translates to:
  /// **'let note = merge(a, b);'**
  String get fontsSampleCode;

  /// The section that sets how large the text of notes is.
  ///
  /// In en, this message translates to:
  /// **'Text size'**
  String get fontsTextSizeSection;

  /// Explains the text size.
  ///
  /// In en, this message translates to:
  /// **'Changes the text of notes only. The rest of Kanso follows your device’s text size.'**
  String get fontsTextSizeFooter;

  /// A paragraph that shows the chosen text size.
  ///
  /// In en, this message translates to:
  /// **'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.'**
  String get fontsTextSizeSample;

  /// A text size.
  ///
  /// In en, this message translates to:
  /// **'Small'**
  String get textSizeSmall;

  /// The text size the app was designed at.
  ///
  /// In en, this message translates to:
  /// **'Default'**
  String get textSizeDefault;

  /// A text size.
  ///
  /// In en, this message translates to:
  /// **'Large'**
  String get textSizeLarge;

  /// The largest text size.
  ///
  /// In en, this message translates to:
  /// **'Larger'**
  String get textSizeLarger;

  /// What the empty title field of a note being edited says.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get editorTitleHint;

  /// Says a note was changed elsewhere while it was being edited.
  ///
  /// In en, this message translates to:
  /// **'This note was changed elsewhere while you were editing. The version you don\'t keep stays in its history.'**
  String get editorChangedMessage;

  /// Keeps the text typed here over a change made elsewhere.
  ///
  /// In en, this message translates to:
  /// **'Keep mine'**
  String get editorKeepMine;

  /// Takes a change made elsewhere over the text typed here.
  ///
  /// In en, this message translates to:
  /// **'Use theirs'**
  String get editorUseTheirs;

  /// Says a note was deleted elsewhere while it was being edited.
  ///
  /// In en, this message translates to:
  /// **'This note was deleted elsewhere while you were editing.'**
  String get editorDeletedMessage;

  /// Keeps a note that was deleted elsewhere, by saving it again.
  ///
  /// In en, this message translates to:
  /// **'Keep it'**
  String get editorKeepIt;

  /// Lets go of a note that was deleted elsewhere.
  ///
  /// In en, this message translates to:
  /// **'Discard'**
  String get editorDiscard;

  /// Says that saving a note failed.
  ///
  /// In en, this message translates to:
  /// **'Your changes aren\'t saved yet.'**
  String get editorSaveFailed;

  /// What a screen reader says while a note is opened for editing.
  ///
  /// In en, this message translates to:
  /// **'Opening the note'**
  String get editorOpeningLabel;

  /// Adds a property to the note.
  ///
  /// In en, this message translates to:
  /// **'Add property'**
  String get propertiesAdd;

  /// The hint of the field for the name of a new property.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get propertiesName;

  /// The hint of the field for the value of a property.
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get propertiesValue;

  /// Removes a property from the note; for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Remove {name}'**
  String propertiesRemove(String name);

  /// Adds the property that was just named.
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get propertiesConfirmAdd;

  /// Gives up on adding a property.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get propertiesCancelAdd;

  /// Says that a property of the note could not be set.
  ///
  /// In en, this message translates to:
  /// **'That property couldn\'t be set.'**
  String get propertiesFailed;

  /// What the row of properties does when it is closed, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Show properties'**
  String get propertiesShow;

  /// What the row of properties does when it is open, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Hide properties'**
  String get propertiesHide;

  /// Names the switch between a rendered note and its text.
  ///
  /// In en, this message translates to:
  /// **'View mode'**
  String get noteModeLabel;

  /// Says to a screen reader that the list of notes is loading.
  ///
  /// In en, this message translates to:
  /// **'Loading notes'**
  String get noteListLoading;

  /// The title of a notebook that has no notes.
  ///
  /// In en, this message translates to:
  /// **'No notes in {notebook} yet'**
  String emptyNotebookTitle(String notebook);

  /// Says what an empty notebook will hold.
  ///
  /// In en, this message translates to:
  /// **'Notes added here will appear in this list.'**
  String get emptyNotebookMessage;

  /// The title of the area where a note is read when none is open.
  ///
  /// In en, this message translates to:
  /// **'No note selected'**
  String get noNoteSelectedTitle;

  /// Says how to open a note.
  ///
  /// In en, this message translates to:
  /// **'Choose a note from the list to read it here.'**
  String get noNoteSelectedMessage;

  /// The library row that opens the gallery of the design proposal; in development builds only.
  ///
  /// In en, this message translates to:
  /// **'Design gallery'**
  String get libraryDesignGallery;

  /// The button that tries a failed reading again.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get retry;

  /// What a screen reader says while the library's rows are read.
  ///
  /// In en, this message translates to:
  /// **'Loading the library'**
  String get libraryLoading;

  /// The title of the card shown when the notebooks and tags could not be read.
  ///
  /// In en, this message translates to:
  /// **'The library didn\'t load'**
  String get libraryFailedTitle;

  /// The message of the card shown when the notebooks and tags could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the notebooks and tags on this device. Try again.'**
  String get libraryFailedMessage;

  /// Shown under the Notebooks heading when the library has no notebooks.
  ///
  /// In en, this message translates to:
  /// **'No notebooks yet'**
  String get libraryNoNotebooks;

  /// Shown under the Tags heading when no note has a tag.
  ///
  /// In en, this message translates to:
  /// **'No tags yet'**
  String get libraryNoTags;

  /// A day in a list's header or a row's time: the day it is.
  ///
  /// In en, this message translates to:
  /// **'Today'**
  String get dayToday;

  /// A day in a list's header or a row's time: the day before.
  ///
  /// In en, this message translates to:
  /// **'Yesterday'**
  String get dayYesterday;

  /// What a note with no title is called in a list.
  ///
  /// In en, this message translates to:
  /// **'Untitled'**
  String get noteUntitled;

  /// The line of text a locked note's row shows in place of its text.
  ///
  /// In en, this message translates to:
  /// **'Locked'**
  String get noteListLocked;

  /// The title of the card shown when a list of notes could not be read.
  ///
  /// In en, this message translates to:
  /// **'These notes didn\'t load'**
  String get noteListFailedTitle;

  /// The message of the card shown when a list of notes could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the notes on this device. Try again.'**
  String get noteListFailedMessage;

  /// The title shown when the library has no notes.
  ///
  /// In en, this message translates to:
  /// **'No notes yet'**
  String get emptyAllTitle;

  /// What the empty list of all notes says will appear there.
  ///
  /// In en, this message translates to:
  /// **'Notes will appear here.'**
  String get emptyAllMessage;

  /// The title shown when no note is pinned.
  ///
  /// In en, this message translates to:
  /// **'Nothing pinned'**
  String get emptyPinnedTitle;

  /// What the empty list of pinned notes says.
  ///
  /// In en, this message translates to:
  /// **'Pin a note by swiping it, and it will wait here.'**
  String get emptyPinnedMessage;

  /// The title shown when no note was changed lately.
  ///
  /// In en, this message translates to:
  /// **'Nothing recent'**
  String get emptyRecentTitle;

  /// What the empty list of recent notes says.
  ///
  /// In en, this message translates to:
  /// **'Notes you have changed lately will appear here.'**
  String get emptyRecentMessage;

  /// The title shown when the trash has no notes.
  ///
  /// In en, this message translates to:
  /// **'The trash is empty'**
  String get emptyTrashTitle;

  /// What the empty trash says.
  ///
  /// In en, this message translates to:
  /// **'Notes you trash wait here until you move them out.'**
  String get emptyTrashMessage;

  /// The title shown when no note has the tag.
  ///
  /// In en, this message translates to:
  /// **'No notes tagged {tag}'**
  String emptyTagTitle(String tag);

  /// What the empty list of a tag's notes says.
  ///
  /// In en, this message translates to:
  /// **'Notes with this tag will appear in this list.'**
  String get emptyTagMessage;

  /// The title shown when the list's narrowing hides every note.
  ///
  /// In en, this message translates to:
  /// **'No notes match'**
  String get emptyNarrowedTitle;

  /// What the list says when its narrowing hides every note.
  ///
  /// In en, this message translates to:
  /// **'Try another status, or turn off Pinned Only.'**
  String get emptyNarrowedMessage;

  /// The title of the sheet that sorts and narrows a list.
  ///
  /// In en, this message translates to:
  /// **'View options'**
  String get listOptionsTitle;

  /// The heading of the choices of what to sort a list by.
  ///
  /// In en, this message translates to:
  /// **'Sort by'**
  String get listOptionsSortBy;

  /// A way to sort a list: by when each note was last changed.
  ///
  /// In en, this message translates to:
  /// **'Date Updated'**
  String get listOptionsDateUpdated;

  /// A way to sort a list: by when each note was made.
  ///
  /// In en, this message translates to:
  /// **'Date Created'**
  String get listOptionsDateCreated;

  /// A way to sort a list: by each note's title.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get listOptionsTitleSort;

  /// A direction of a list sorted by date.
  ///
  /// In en, this message translates to:
  /// **'Newest First'**
  String get listOptionsNewestFirst;

  /// A direction of a list sorted by date.
  ///
  /// In en, this message translates to:
  /// **'Oldest First'**
  String get listOptionsOldestFirst;

  /// A direction of a list sorted by title.
  ///
  /// In en, this message translates to:
  /// **'A to Z'**
  String get listOptionsAToZ;

  /// A direction of a list sorted by title.
  ///
  /// In en, this message translates to:
  /// **'Z to A'**
  String get listOptionsZToA;

  /// The heading of the choices of a status to narrow a list to.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get listOptionsStatus;

  /// The choice that does not narrow a list by status.
  ///
  /// In en, this message translates to:
  /// **'Any Status'**
  String get listOptionsAnyStatus;

  /// The choice that narrows a list to pinned notes.
  ///
  /// In en, this message translates to:
  /// **'Pinned Only'**
  String get listOptionsPinnedOnly;

  /// The button that ends choosing several notes, or closes a sheet.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get noteListDone;

  /// How many notes are chosen.
  ///
  /// In en, this message translates to:
  /// **'{count} selected'**
  String noteListSelected(int count);

  /// The action that pins the notes chosen.
  ///
  /// In en, this message translates to:
  /// **'Pin'**
  String get noteListPin;

  /// The action that unpins the notes chosen.
  ///
  /// In en, this message translates to:
  /// **'Unpin'**
  String get noteListUnpin;

  /// The action that moves notes to a notebook.
  ///
  /// In en, this message translates to:
  /// **'Move'**
  String get noteListMove;

  /// The action that moves notes to the trash.
  ///
  /// In en, this message translates to:
  /// **'Trash'**
  String get noteListTrash;

  /// The title of the sheet that asks which notebook to move notes to.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Move 1 note to} other{Move {count} notes to}}'**
  String moveSheetTitle(int count);

  /// The button that closes a sheet without doing what it offers.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get noteListCancel;

  /// The line shown when a change to notes did not happen.
  ///
  /// In en, this message translates to:
  /// **'That didn\'t work. Try again.'**
  String get noteListChangeFailed;

  /// The line shown when notes could not be moved into or out of a locked notebook.
  ///
  /// In en, this message translates to:
  /// **'Locked notebooks can\'t take or give up notes here yet.'**
  String get noteListChangeLocked;

  /// What the add button of the notebooks header does.
  ///
  /// In en, this message translates to:
  /// **'New notebook'**
  String get libraryNewNotebook;

  /// The title of the sheet that makes a notebook.
  ///
  /// In en, this message translates to:
  /// **'New notebook'**
  String get newNotebookTitle;

  /// The hint, and the label, of the field for a new notebook's name.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get newNotebookNameHint;

  /// The button that makes the notebook.
  ///
  /// In en, this message translates to:
  /// **'Create'**
  String get newNotebookCreate;

  /// The line shown when a new notebook's name is empty or not accepted.
  ///
  /// In en, this message translates to:
  /// **'Give the notebook a name.'**
  String get newNotebookInvalid;

  /// The line shown when a notebook could not be made.
  ///
  /// In en, this message translates to:
  /// **'The notebook couldn\'t be made. Try again.'**
  String get newNotebookFailed;

  /// What the add button of the note list does.
  ///
  /// In en, this message translates to:
  /// **'New note'**
  String get noteListNewNote;

  /// The line shown when a new note was asked for in a notebook that is locked.
  ///
  /// In en, this message translates to:
  /// **'Unlock this notebook to add notes to it.'**
  String get noteListNewNoteLocked;

  /// The line shown when a new note could not be made.
  ///
  /// In en, this message translates to:
  /// **'The note couldn\'t be made. Try again.'**
  String get noteListNewNoteFailed;

  /// The button that closes the line that says a change did not happen.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get noteListDismiss;

  /// What a screen reader says while a note is read.
  ///
  /// In en, this message translates to:
  /// **'Loading the note'**
  String get noteLoading;

  /// The title shown when a note that was asked for does not exist.
  ///
  /// In en, this message translates to:
  /// **'This note isn\'t here'**
  String get noteNotFoundTitle;

  /// What the screen of a note that is not there says.
  ///
  /// In en, this message translates to:
  /// **'It may have been deleted, or it hasn\'t reached this device yet.'**
  String get noteNotFoundMessage;

  /// The title of the card shown when a note could not be read.
  ///
  /// In en, this message translates to:
  /// **'This note didn\'t load'**
  String get noteFailedTitle;

  /// The message of the card shown when a note could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the note on this device. Try again.'**
  String get noteFailedMessage;

  /// What the card of a locked note says about its text.
  ///
  /// In en, this message translates to:
  /// **'Its text stays sealed. Only its title can be shown here.'**
  String get lockedMessageSealed;

  /// The button of a text field's menu that cuts the selected text.
  ///
  /// In en, this message translates to:
  /// **'Cut'**
  String get menuCut;

  /// The button of a text field's menu that copies the selected text.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get menuCopy;

  /// The button of a text field's menu that pastes what was copied.
  ///
  /// In en, this message translates to:
  /// **'Paste'**
  String get menuPaste;

  /// The button of a text field's menu that selects all of its text.
  ///
  /// In en, this message translates to:
  /// **'Select All'**
  String get menuSelectAll;

  /// The button of the spelling menu that deletes the misspelled word.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get menuDelete;

  /// The button of the editor's menu that opens the link the caret is in.
  ///
  /// In en, this message translates to:
  /// **'Open link'**
  String get menuOpenLink;

  /// What a screen reader says when suggestions for a link or a tag appear: how many, and which one is selected.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 suggestion} other{{count} suggestions}}, {label} selected'**
  String completionAnnouncement(int count, String label);

  /// The title shown under an empty search.
  ///
  /// In en, this message translates to:
  /// **'Search your notes'**
  String get searchIdleTitle;

  /// What an empty search says it finds.
  ///
  /// In en, this message translates to:
  /// **'Find words in titles and text, or a tag.'**
  String get searchIdleMessage;

  /// The title shown when a search finds nothing.
  ///
  /// In en, this message translates to:
  /// **'No notes match'**
  String get searchNoResultsTitle;

  /// What a search that finds nothing says.
  ///
  /// In en, this message translates to:
  /// **'Nothing matched “{query}”. Try fewer or different words.'**
  String searchNoResultsMessage(String query);

  /// The title of the card shown when a search failed.
  ///
  /// In en, this message translates to:
  /// **'The search didn\'t run'**
  String get searchFailedTitle;

  /// The message of the card shown when a search failed.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong searching the notes on this device. Try again.'**
  String get searchFailedMessage;

  /// What a screen reader says while the notes are searched.
  ///
  /// In en, this message translates to:
  /// **'Searching'**
  String get searchLoading;

  /// The Theme row's value while the light and the dark theme follow the system: the names of both.
  ///
  /// In en, this message translates to:
  /// **'{light}, {dark}'**
  String settingsThemeBoth(String light, String dark);

  /// The label of the row that shows the state of sync.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get settingsSyncStatus;

  /// The state of sync while no account is signed in.
  ///
  /// In en, this message translates to:
  /// **'Off'**
  String get settingsSyncOff;

  /// The state of sync when an account is signed in and nothing has been synced yet.
  ///
  /// In en, this message translates to:
  /// **'On'**
  String get settingsSyncOn;

  /// The state of sync while it runs.
  ///
  /// In en, this message translates to:
  /// **'Syncing…'**
  String get settingsSyncSyncing;

  /// The state of sync when it has finished.
  ///
  /// In en, this message translates to:
  /// **'Synced'**
  String get settingsSyncSynced;

  /// The state of sync when it could not finish.
  ///
  /// In en, this message translates to:
  /// **'Needs attention'**
  String get settingsSyncAttention;

  /// Says under Sync that device backups do not hold the notes.
  ///
  /// In en, this message translates to:
  /// **'The notes on this device aren’t included in device backups.'**
  String get settingsSyncBackupFooter;

  /// The app's version and the number of its build.
  ///
  /// In en, this message translates to:
  /// **'{version} ({build})'**
  String settingsVersionValue(String version, String build);

  /// Says to a screen reader that the licences are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading licences'**
  String get licencesLoading;

  /// The title of the card shown when the licences could not be read.
  ///
  /// In en, this message translates to:
  /// **'The licences didn’t load'**
  String get licencesFailedTitle;

  /// The message of the card shown when the licences could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the licences. Try again.'**
  String get licencesFailedMessage;

  /// The title shown when there are no licences to list.
  ///
  /// In en, this message translates to:
  /// **'No licences'**
  String get licencesEmptyTitle;

  /// Says what the empty list of licences will hold.
  ///
  /// In en, this message translates to:
  /// **'The packages Kanso is made of will be listed here.'**
  String get licencesEmptyMessage;

  /// How many licences a package comes under.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 licence} other{{count} licences}}'**
  String licencesCount(int count);

  /// The switch that makes the interface follow the system's light or dark mode.
  ///
  /// In en, this message translates to:
  /// **'Switch between a light and a dark theme with the system'**
  String get themesFollowSystem;

  /// Says what following the system does, while it is on.
  ///
  /// In en, this message translates to:
  /// **'Showing the light theme in light mode and the dark one in dark mode.'**
  String get themesFollowSystemOn;

  /// Says what happens while following the system is off.
  ///
  /// In en, this message translates to:
  /// **'Off: the theme stays as picked whatever the system’s mode.'**
  String get themesFollowSystemOff;

  /// The title of the card shown when the themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'The themes didn’t load'**
  String get themesFailedTitle;

  /// The message of the card shown when the themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the themes. Try again.'**
  String get themesFailedMessage;

  /// Says to a screen reader that the themes are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading themes'**
  String get themesLoading;

  /// The title of the card that says the notes are not in device backups.
  ///
  /// In en, this message translates to:
  /// **'Notes stay on this device'**
  String get backupNoticeTitle;

  /// Says that device backups do not hold the notes.
  ///
  /// In en, this message translates to:
  /// **'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.'**
  String get backupNoticeMessage;

  /// Dismisses the notice about device backups.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get backupNoticeOk;

  /// The app's version, the number of its build and the number of the Shorebird patch it is running.
  ///
  /// In en, this message translates to:
  /// **'{version} ({build}), patch {patch}'**
  String settingsVersionPatchValue(String version, String build, int patch);

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Privacy'**
  String get settingsPrivacy;

  /// Says under Privacy that nothing is sent on its own.
  ///
  /// In en, this message translates to:
  /// **'Kanso sends a crash report only when you choose to.'**
  String get settingsPrivacyFooter;

  /// The settings row and the page that list the crash reports waiting on the device.
  ///
  /// In en, this message translates to:
  /// **'Crash reports'**
  String get crashReportsTitle;

  /// Says to a screen reader that the crash reports are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading crash reports'**
  String get crashReportsLoading;

  /// The title of the page when no crash report is waiting.
  ///
  /// In en, this message translates to:
  /// **'No crash reports'**
  String get crashReportsEmptyTitle;

  /// Says what will appear on the Crash reports page.
  ///
  /// In en, this message translates to:
  /// **'If Kanso closes unexpectedly, a report waits here until you decide whether to send it.'**
  String get crashReportsEmptyMessage;

  /// Explains, above the list, what a crash report holds.
  ///
  /// In en, this message translates to:
  /// **'A report holds only where in Kanso’s code something went wrong. It never holds note text, titles or file names. Nothing is sent unless you send it.'**
  String get crashReportsIntro;

  /// Sends every crash report waiting.
  ///
  /// In en, this message translates to:
  /// **'Send all'**
  String get crashReportsSendAll;

  /// A crash report's row: the day, and where it came from.
  ///
  /// In en, this message translates to:
  /// **'{day} · {kind}'**
  String crashReportSubtitle(String day, String kind);

  /// Sends one crash report.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get crashReportSend;

  /// Deletes one crash report without sending it.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get crashReportDelete;

  /// Shows the exact text of a crash report.
  ///
  /// In en, this message translates to:
  /// **'Show report'**
  String get crashReportShow;

  /// Hides the text of a crash report.
  ///
  /// In en, this message translates to:
  /// **'Hide report'**
  String get crashReportHide;

  /// Says a crash report is being sent.
  ///
  /// In en, this message translates to:
  /// **'Sending…'**
  String get crashReportSending;

  /// The origin of a crash report: an error in Kanso's own code.
  ///
  /// In en, this message translates to:
  /// **'App'**
  String get crashKindDart;

  /// The origin of a crash report: a crash the system recorded on iPhone or iPad.
  ///
  /// In en, this message translates to:
  /// **'iOS'**
  String get crashKindIosNative;

  /// The origin of a crash report: a crash in native code on Android.
  ///
  /// In en, this message translates to:
  /// **'Android native'**
  String get crashKindAndroidNative;

  /// The origin of a crash report: the app stopped responding on Android.
  ///
  /// In en, this message translates to:
  /// **'Android not responding'**
  String get crashKindAndroidAnr;

  /// The origin of a crash report: an uncaught exception on Android.
  ///
  /// In en, this message translates to:
  /// **'Android'**
  String get crashKindAndroidJvm;

  /// The title of the card shown after a launch that found crash reports.
  ///
  /// In en, this message translates to:
  /// **'Kanso closed unexpectedly'**
  String get crashNoticeTitle;

  /// Asks whether to send crash reports and says what they hold.
  ///
  /// In en, this message translates to:
  /// **'Send a crash report? It holds only where in Kanso’s code it happened, never note text, titles or file names.'**
  String get crashNoticeMessage;

  /// Sends the crash reports.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get crashNoticeSend;

  /// Deletes the crash reports without sending them.
  ///
  /// In en, this message translates to:
  /// **'Don’t send'**
  String get crashNoticeDiscard;

  /// Said after a crash report was sent.
  ///
  /// In en, this message translates to:
  /// **'Report sent. Thank you.'**
  String get crashOutcomeSent;

  /// Said when the server did not accept a crash report.
  ///
  /// In en, this message translates to:
  /// **'The report couldn’t be sent.'**
  String get crashOutcomeRejected;

  /// Said when the server asked to wait.
  ///
  /// In en, this message translates to:
  /// **'Too many reports just now. Try again later.'**
  String get crashOutcomeTryLater;

  /// Said when a crash report could not be sent for lack of a connection.
  ///
  /// In en, this message translates to:
  /// **'Couldn’t reach Kanso. Try again later.'**
  String get crashOutcomeUnreachable;

  /// The message of the card shown when the crash reports could not be read.
  ///
  /// In en, this message translates to:
  /// **'The crash reports couldn’t be read.'**
  String get crashReportsFailedMessage;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
