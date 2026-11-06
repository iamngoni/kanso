# Overview

Overview is Kanso's home. **Today** answers what needs attention today; **This week** gathers recent
work and what is ahead. Both views use the same date header and are useful without AI.

## Today

The agenda combines system-calendar events, timed reminders, tasks due today and overdue tasks in time
order. All-day events appear with the date. The agenda folds earlier events and caps the rows it shows.
Tasks can be completed from the page. Calendar events are read-only.

After the agenda, Overview can show the notes you were last writing, items waiting on a person with a
People page, shared and agent activity, notes due to revisit, recent Inbox captures, high-confidence
promises, and one related note from the past. Empty sections stay hidden, lists have a three-row cap,
and the third list folds into the secondary lines.

## This week

The week view groups active projects, finished work, items still open, and the coming days. It also
shows weekly themes, meetings without notes, notes due to revisit and Inbox counts when those have
content. A week can be saved as a Journal note.

## Calendar privacy

Kanso reads only calendars configured in the device's system calendar: EventKit on iOS and macOS, and
CalendarContract on Android. A Google or Microsoft account is available when it is already configured
with the system calendar. Each calendar has its own switch. Kanso never writes to a calendar, sends
events to its server or connects directly to a calendar provider or calendar address.

Calendar access is optional. If permission is missing or denied, Overview works without calendar rows.
Permission is requested from **Settings › Calendar**, where the reason is explained before the
operating system prompt appears.

## AI paragraph

When AI is enabled, **Summarise** asks for a short paragraph for the current view. The paragraph is
cached locally and rendered as Markdown. A note link is active only when the note is among the sources
cited for that paragraph. Calendar event titles and times are not sent to the model by default.

## Older names

Morning Brief and Weekly Review now open Overview's Today and This week views. Their notification
settings and saved weekly notes keep their existing data.
