package app.kanso.kanso

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.database.Cursor
import android.provider.CalendarContract
import android.net.Uri
import android.content.SharedPreferences
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/** Read-only access to Android's system calendars for Overview. */
object KansoCalendar {
    private const val channelName = "app.kanso.kanso/system_calendar"
    private const val permissionRequest = 2104
    private var pendingPermission: MethodChannel.Result? = null

    fun register(activity: Activity, engine: FlutterEngine) {
        MethodChannel(engine.dartExecutor.binaryMessenger, channelName)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "requestAccess" -> requestAccess(activity, result)
                    "calendars" -> withAccess(activity, result) { calendars(activity) }
                    "setCalendarEnabled" -> {
                        val id = call.argument<String>("id")
                        val enabled = call.argument<Boolean>("enabled")
                        if (id.isNullOrBlank() || enabled == null) {
                            result.error("calendar.selection", "Invalid calendar selection.", null)
                        } else {
                            val disabled = disabled(activity).toMutableSet()
                            if (enabled) disabled.remove(id) else disabled.add(id)
                            preferences(activity).edit().putStringSet("disabled", disabled).apply()
                            result.success(null)
                        }
                    }
                    "openEvent" -> {
                        val start = call.argument<Number>("start")?.toLong()
                        if (start == null) {
                            result.error("calendar.event", "Invalid calendar event.", null)
                        } else {
                            try {
                                val intent = Intent(
                                    Intent.ACTION_VIEW,
                                    Uri.parse("content://com.android.calendar/time/$start"),
                                ).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                activity.startActivity(intent)
                                result.success(null)
                            } catch (error: Exception) {
                                result.error("calendar.open", "Couldn't open the event in Calendar.", null)
                            }
                        }
                    }
                    "events" -> {
                        val start = call.argument<Number>("start")?.toLong()
                        val end = call.argument<Number>("end")?.toLong()
                        if (start == null || end == null || start >= end) {
                            result.error("calendar.range", "Invalid calendar range.", null)
                        } else {
                            withAccess(activity, result) { events(activity, start, end) }
                        }
                    }
                    else -> result.notImplemented()
                }
            }
    }

    private fun requestAccess(activity: Activity, result: MethodChannel.Result) {
        if (hasAccess(activity)) {
            result.success(true)
            return
        }
        if (pendingPermission != null) {
            result.error("calendar.pending", "Calendar permission is already being requested.", null)
            return
        }
        pendingPermission = result
        ActivityCompat.requestPermissions(
            activity,
            arrayOf(Manifest.permission.READ_CALENDAR),
            permissionRequest,
        )
    }

    fun onRequestPermissionsResult(requestCode: Int, grantResults: IntArray) {
        if (requestCode != permissionRequest) return
        val result = pendingPermission ?: return
        pendingPermission = null
        result.success(grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED)
    }

    private fun hasAccess(activity: Activity): Boolean =
        ContextCompat.checkSelfPermission(activity, Manifest.permission.READ_CALENDAR) ==
            PackageManager.PERMISSION_GRANTED

    private fun <T> withAccess(activity: Activity, result: MethodChannel.Result, body: () -> T) {
        if (!hasAccess(activity)) {
            result.error("calendar.permission", "Allow calendar access in Settings to read events.", null)
            return
        }
        try {
            result.success(body())
        } catch (error: SecurityException) {
            result.error("calendar.permission", "Calendar access is unavailable.", null)
        } catch (error: Exception) {
            result.error("calendar.read", "Couldn't read system calendars.", null)
        }
    }

    private fun calendars(activity: Activity): List<Map<String, Any?>> {
        val columns = arrayOf(
            CalendarContract.Calendars._ID,
            CalendarContract.Calendars.CALENDAR_DISPLAY_NAME,
            CalendarContract.Calendars.CALENDAR_COLOR,
            CalendarContract.Calendars.VISIBLE,
        )
        val disabledIds = disabled(activity)
        return activity.contentResolver.query(
            CalendarContract.Calendars.CONTENT_URI,
            columns,
            "${CalendarContract.Calendars.VISIBLE}=1",
            null,
            CalendarContract.Calendars.CALENDAR_DISPLAY_NAME,
        )?.use { cursor ->
            buildList {
                while (cursor.moveToNext()) {
                    val id = cursor.string(CalendarContract.Calendars._ID)
                    add(
                        mapOf(
                            "id" to id,
                            "name" to cursor.string(CalendarContract.Calendars.CALENDAR_DISPLAY_NAME),
                            "color" to cursor.intOrNull(CalendarContract.Calendars.CALENDAR_COLOR)
                                ?.let(::colorHex).orEmpty(),
                            "enabled" to !disabledIds.contains(id),
                        ),
                    )
                }
            }
        } ?: emptyList()
    }

    private fun events(activity: Activity, start: Long, end: Long): List<Map<String, Any?>> {
        val uri = CalendarContract.Instances.CONTENT_URI.buildUpon()
            .appendPath(start.toString())
            .appendPath(end.toString())
            .build()
        val columns = arrayOf(
            CalendarContract.Instances.EVENT_ID,
            CalendarContract.Instances.BEGIN,
            CalendarContract.Instances.END,
            CalendarContract.Instances.TITLE,
            CalendarContract.Instances.EVENT_LOCATION,
            CalendarContract.Instances.CALENDAR_ID,
            CalendarContract.Instances.CALENDAR_DISPLAY_NAME,
            CalendarContract.Instances.CALENDAR_COLOR,
            CalendarContract.Instances.ALL_DAY,
            CalendarContract.Instances.AVAILABILITY,
            CalendarContract.Instances.SELF_ATTENDEE_STATUS,
        )
        val disabledIds = disabled(activity)
        val cursor = activity.contentResolver.query(
            uri,
            columns,
            null,
            null,
            "${CalendarContract.Instances.BEGIN} ASC",
        ) ?: return emptyList()
        return cursor.use { rows ->
            val events = mutableListOf<Map<String, Any?>>()
            while (rows.moveToNext()) {
                if (disabledIds.contains(rows.string(CalendarContract.Instances.CALENDAR_ID))) {
                    continue
                }
                events.add(
                    mapOf(
                        "uid" to rows.string(CalendarContract.Instances.EVENT_ID),
                        "start" to rows.long(CalendarContract.Instances.BEGIN),
                        "end" to rows.long(CalendarContract.Instances.END),
                        "title" to rows.string(CalendarContract.Instances.TITLE),
                        "location" to rows.string(CalendarContract.Instances.EVENT_LOCATION),
                        "attendees" to attendees(activity, rows.string(CalendarContract.Instances.EVENT_ID)),
                        "calendarId" to rows.string(CalendarContract.Instances.CALENDAR_ID),
                        "calendar" to rows.string(CalendarContract.Instances.CALENDAR_DISPLAY_NAME),
                        "color" to rows.intOrNull(CalendarContract.Instances.CALENDAR_COLOR)
                            ?.let(::colorHex).orEmpty(),
                        "allDay" to (rows.int(CalendarContract.Instances.ALL_DAY) != 0),
                        "free" to (rows.int(CalendarContract.Instances.AVAILABILITY) ==
                            CalendarContract.Events.AVAILABILITY_FREE),
                        "declined" to (rows.int(CalendarContract.Instances.SELF_ATTENDEE_STATUS) ==
                            CalendarContract.Attendees.ATTENDEE_STATUS_DECLINED),
                    ),
                )
            }
            events
        }
    }

    private fun attendees(activity: Activity, eventId: String): List<Map<String, String>> {
        if (eventId.isBlank()) return emptyList()
        val columns = arrayOf(
            CalendarContract.Attendees.ATTENDEE_NAME,
            CalendarContract.Attendees.ATTENDEE_EMAIL,
        )
        return activity.contentResolver.query(
            CalendarContract.Attendees.CONTENT_URI,
            columns,
            "${CalendarContract.Attendees.EVENT_ID}=?",
            arrayOf(eventId),
            CalendarContract.Attendees.ATTENDEE_NAME,
        )?.use { cursor ->
            buildList {
                while (cursor.moveToNext()) {
                    val name = cursor.string(CalendarContract.Attendees.ATTENDEE_NAME)
                    val email = cursor.string(CalendarContract.Attendees.ATTENDEE_EMAIL)
                    if (name.isNotBlank() || email.isNotBlank()) {
                        add(mapOf("name" to name, "email" to email))
                    }
                }
            }
        } ?: emptyList()
    }

    private fun Cursor.string(column: String): String = getColumnIndex(column)
        .takeIf { it >= 0 && !isNull(it) }
        ?.let { getString(it) }
        ?: ""

    private fun Cursor.int(column: String): Int = getColumnIndex(column)
        .takeIf { it >= 0 && !isNull(it) }
        ?.let { getInt(it) }
        ?: 0

    private fun Cursor.long(column: String): Long = getColumnIndex(column)
        .takeIf { it >= 0 && !isNull(it) }
        ?.let { getLong(it) }
        ?: 0L

    private fun Cursor.intOrNull(column: String): Int? = getColumnIndex(column)
        .takeIf { it >= 0 && !isNull(it) }
        ?.let { getInt(it) }

    private fun colorHex(color: Int): String = "#%06x".format(color and 0x00ffffff)

    private fun preferences(activity: Activity): SharedPreferences =
        activity.getSharedPreferences("overview.calendar", Activity.MODE_PRIVATE)

    private fun disabled(activity: Activity): Set<String> =
        preferences(activity).getStringSet("disabled", emptySet())?.toSet() ?: emptySet()
}
