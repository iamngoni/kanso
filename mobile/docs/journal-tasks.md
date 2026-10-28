# Journal, templates, reminders, boards and due chips (#48 packages F and G)

The mobile side of what the desktop does in `crates/kanso-app/src/journal.rs`,
`calendar.rs`, `calendar_view.rs`, `settings_capture.rs` (the daily template),
`reminders.rs`, `board.rs`, `board_view.rs` and `due.rs`. Every decision is
made in Rust (`kanso_engine::{daily, calendar, board, tasks}`), which the
desktop calls too; `crates/kanso-mobile` hands it to Dart, and Flutter only
draws and schedules.

## The bridge (`mobile/rust/src/api/journal.rs`, `board.rs`, `reminders.rs`)

All methods are on `KansoLibrary`. Times cross the bridge as wall-clock
`LocalTime` (the same struct step 10 uses for Today's note); text offsets as
UTF-16 code units.

```rust
pub struct LocalTime { year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32 }
pub struct DayDate { year: i32, month: u32, day: u32 }

/// A note found or made, for the app to open.
pub struct MadeNote { id: String, cursor16: Option<u32>, title_first: bool }

// Journal
fn journal_today(&self, now: LocalTime) -> Result<MadeNote>
fn journal_step(&self, open_note: Option<String>, now: LocalTime, forward: bool) -> Result<Option<MadeNote>>
fn journal_open_day(&self, day: DayDate, now: LocalTime) -> Result<MadeNote>
fn journal_month(&self, year: i32, month: u32, week_start: u32, today: DayDate) -> Result<MonthGrid>
fn daily_date_of(note_id: String) -> Option<DayDate>        // associated, no library, sync
fn journal_week_start(first_weekday: u32) -> u32            // associated, sync: Foundation's/Java's
                                                            // 1 = Sunday .. 7 = Saturday -> 0 = Monday .. 6 = Sunday

pub struct MonthGrid { year: i32, month: u32, title: String, weekday_labels: Vec<String>, weeks: Vec<Vec<CalendarDay>> }
pub struct CalendarDay { date: DayDate, in_month: bool, has_note: bool, is_today: bool }

// Templates
fn templates(&self) -> Result<Vec<TemplateRow>>               // TemplateRow { id, title }
fn new_from_template(&self, template_id: String, book_id: Option<String>, now: LocalTime) -> Result<MadeNote>
fn new_template(&self) -> Result<MadeNote>
fn daily_template(&self) -> Result<Option<String>>
fn set_daily_template(&self, template_id: Option<String>) -> Result<()>

// Boards: over the notes a ListQuery lists (the trash never shows)
pub enum BoardKind { Notes, Tasks }
pub enum BoardLane { Status { status: String }, Todo, Done }   // status as NoteRow.status says it:
                                                               // none, active, onHold, completed, dropped
fn board(&self, kind: BoardKind, query: ListQuery) -> Result<BoardView>
fn move_card(&self, card: CardKey, to: BoardLane, now_ms: i64) -> Result<CardMoved>
pub struct BoardView { kind, columns: Vec<BoardColumn> }
pub struct BoardColumn { lane: BoardLane, label: String, cards: Vec<BoardCard> }
pub struct BoardCard { key: CardKey, title: String, note_title: String, snippet: String, due: Option<String>, checked: bool, locked: bool }
pub enum CardKey { Note { id }, Task { note_id, line, text } }
pub enum CardMoved { Changed, Unchanged, Missing }
fn due_chip(due: String, checked: bool, now: LocalTime) -> Result<Option<DueChipRow>>   // associated, sync
pub struct DueChipRow { label: String, tone: DueTone, tooltip: String }
pub enum DueTone { Plain, Today, Overdue, Done }

// Reminders
fn reminders_plan(&self, now_utc_ms: i64, offset_ms: i64, can_notify: bool) -> Result<ReminderPlan>
fn reminder_answer(&self, key: String, answer: ReminderAnswer, now_utc_ms: i64) -> Result<Option<TaskPlace>>
pub struct ReminderPlan { show_now: Vec<ReminderNote>, schedule: Vec<ReminderNote> }
pub struct ReminderNote { key: String, note_id: String, line: u32, title: String, body: String, fire_at: LocalTime }
pub enum ReminderAnswer { Open, MarkDone, Dismiss }
pub struct TaskPlace { note_id: String, line: u32 }
```

`reminders_plan` returns nothing (and records nothing) when `can_notify` is
false: reminders are off in Settings or the OS said no. Otherwise it records
`show_now` as shown now and `schedule` as shown at their fire time, so a
reminder the OS already showed at its time is not shown again when the app
next opens. `schedule` holds at most 60 reminders within the next 30 days,
soonest first (iOS keeps 64 pending notifications per app).

The record of what this device has shown (`app.remindersShown`, local, never
synced) holds a scheduled reminder under its fire time, so a plan made after
that time doesn't show it again; one scheduled for a time still to come counts
as to come, not shown, and is in `schedule` again. A reminder the system never
showed because notifications were off when it fell due is not shown late if it
was scheduled (this keeps one reminder from showing twice; the desktop has no
such gap because it shows the reminder itself).

A reminder carries only what the desktop's notification shows: the task's
words as a list shows them (`tasks::display_text`) as the title, and "when ·
note title" as the body, plus the key, note ID and line. Never the note's
text. Locked notes and the trash have no reminders
(`Store::reminder_tasks` leaves them out); a note locked after a reminder was
scheduled loses it at the next plan, which runs on every library change.

## The native side: the `app.kanso/reminders` method channel

Swift (`ios/Runner/KansoReminders.swift`, `UNUserNotificationCenter`) and
Kotlin (`android/.../KansoReminders.kt`, `AlarmManager` + `NotificationManager`).
A reminder map is `{key, noteId, line, title, body, year, month, day, hour, minute}`
(fire time in the device's local time zone).

| Dart → native | Answer |
| --- | --- |
| `permission` | `"granted"`, `"denied"` or `"notDetermined"` |
| `requestPermission` | `bool` |
| `schedule(List<reminder>)` | replaces every pending Kanso reminder with these |
| `show(List<reminder>)` | shows these now |
| `cancelAll` | removes pending and delivered Kanso reminders |
| `takeResponses` | `List<{key, noteId, line, action}>` answered since the last call, then forgets them |

| Native → Dart | When |
| --- | --- |
| `response {key, noteId, line, action}` | a reminder (or a button) was tapped while the app runs |

`action` is `"open"` (the reminder itself), `"mark-done"` or `"dismiss"`, the
desktop's `kanso_engine::tasks::reminders::{MARK_DONE, DISMISS}`. Mark Done and
Dismiss don't bring the app forward; their answers are kept (UserDefaults /
SharedPreferences) until `takeResponses`, which the app calls at launch and
when it comes back to the foreground. The notification category / channel is
"Reminders", with the buttons "Mark Done" and "Dismiss".

## Due chips in the preview

`kanso_engine::tasks::rewrite_for_preview` writes each due date as a
`kanso-due:` link; `kanso_render::html` draws such a link as
`<span class="kanso-due kanso-due-{tone}" title="Due …">Today 09:00</span>`
when `PageOptions::due` is given, with the chip colours in the page's
stylesheet. `Renderer::preview_html_at(id, palette, now: LocalTime)` does both. Chips read
"Today" and "Overdue" against a clock, so the library needs the phone's local
time: `preview_html(id, palette)` has none and leaves each date as written
(the preview of step 7 calls `previewHtmlAt` to get chips).
