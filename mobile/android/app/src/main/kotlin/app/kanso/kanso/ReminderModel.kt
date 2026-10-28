package app.kanso.kanso

import java.time.LocalDateTime
import java.time.ZoneId
import org.json.JSONArray
import org.json.JSONObject

/**
 * A reminder as Dart sends it over `app.kanso/reminders`
 * (docs/journal-tasks.md), and the little that is kept of it: the words and
 * the fire time (in the device's local time zone) so alarms can be set again
 * after a reboot. Nothing else of the note is ever stored.
 */
data class Reminder(
    val key: String,
    val noteId: String,
    val line: Int,
    val title: String,
    val body: String,
    val year: Int,
    val month: Int,
    val day: Int,
    val hour: Int,
    val minute: Int,
) {
    /** When it fires, in epoch milliseconds, for the time zone [zone]. */
    fun triggerAtMillis(zone: ZoneId = ZoneId.systemDefault()): Long =
        LocalDateTime.of(year, month, day, hour, minute)
            .atZone(zone)
            .toInstant()
            .toEpochMilli()

    fun toJson(): JSONObject =
        JSONObject()
            .put("key", key)
            .put("noteId", noteId)
            .put("line", line)
            .put("title", title)
            .put("body", body)
            .put("year", year)
            .put("month", month)
            .put("day", day)
            .put("hour", hour)
            .put("minute", minute)

    companion object {
        /** From a channel map or a stored object; null when it is malformed. */
        fun fromMap(map: Map<*, *>): Reminder? {
            fun int(name: String) = (map[name] as? Number)?.toInt()
            return try {
                val reminder = Reminder(
                    key = map["key"] as? String ?: return null,
                    noteId = map["noteId"] as? String ?: return null,
                    line = int("line") ?: return null,
                    title = map["title"] as? String ?: "",
                    body = map["body"] as? String ?: "",
                    year = int("year") ?: return null,
                    month = int("month") ?: return null,
                    day = int("day") ?: return null,
                    hour = int("hour") ?: return null,
                    minute = int("minute") ?: return null,
                )
                // A date that doesn't exist (month 13) is a malformed one.
                reminder.triggerAtMillis()
                reminder
            } catch (_: java.time.DateTimeException) {
                null
            }
        }

        fun fromJson(json: JSONObject): Reminder? = fromMap(json.toMap())
    }
}

/** What a tap on a reminder or one of its buttons says. */
data class Answer(
    val key: String,
    val noteId: String,
    val line: Int,
    val action: String,
) {
    /** The map the channel carries. */
    fun toMap(): Map<String, Any> =
        mapOf("key" to key, "noteId" to noteId, "line" to line, "action" to action)

    fun toJson(): JSONObject =
        JSONObject().put("key", key).put("noteId", noteId).put("line", line).put("action", action)

    companion object {
        fun fromJson(json: JSONObject): Answer? {
            val key = json.optString("key", "")
            val noteId = json.optString("noteId", "")
            val action = json.optString("action", "")
            if (key.isEmpty() || noteId.isEmpty() || action.isEmpty() || !json.has("line")) {
                return null
            }
            return Answer(key, noteId, json.optInt("line"), action)
        }
    }
}

/** The rules of the reminders that don't need Android to run. */
object ReminderRules {
    const val OPEN = "open"
    const val MARK_DONE = "mark-done"
    const val DISMISS = "dismiss"

    /** The notification id for a reminder, the same for the same key. */
    fun notificationId(key: String): Int = key.hashCode()

    /** A request code for a `PendingIntent` of [kind] for [key]. */
    fun requestCode(kind: String, key: String): Int = "$kind:$key".hashCode()

    fun encodeReminders(reminders: List<Reminder>): String =
        JSONArray(reminders.map { it.toJson() }).toString()

    /** What [encodeReminders] wrote; anything unreadable is left out. */
    fun decodeReminders(text: String?): List<Reminder> =
        decode(text) { Reminder.fromJson(it) }

    fun encodeAnswers(answers: List<Answer>): String =
        JSONArray(answers.map { it.toJson() }).toString()

    fun decodeAnswers(text: String?): List<Answer> =
        decode(text) { Answer.fromJson(it) }

    private fun <T> decode(text: String?, read: (JSONObject) -> T?): List<T> {
        if (text.isNullOrEmpty()) {
            return emptyList()
        }
        return try {
            val array = JSONArray(text)
            (0 until array.length()).mapNotNull { index ->
                array.optJSONObject(index)?.let(read)
            }
        } catch (_: org.json.JSONException) {
            emptyList()
        }
    }

    /** The reminders still ahead of [now], soonest first. */
    fun upcoming(reminders: List<Reminder>, now: Long): List<Reminder> =
        reminders.filter { it.triggerAtMillis() > now }.sortedBy { it.triggerAtMillis() }

    /**
     * One answer goes one way, never both: live over the channel when the
     * engine is up and can take it, otherwise kept until `takeResponses`.
     * A tap on the notification brings the app forward, so it goes live
     * whenever the engine is up; a button goes live only when the app is in
     * front, as a background process may not get to read it.
     */
    fun deliverLive(action: String, engineUp: Boolean, appInFront: Boolean): Boolean {
        if (!engineUp) {
            return false
        }
        return action == OPEN || appInFront
    }

    /**
     * `granted`, `denied` or `notDetermined`. From Android 13 the runtime
     * permission decides, and "never asked" is told from "refused" by
     * whether it was asked; before that, only whether notifications are on.
     */
    fun permission(sdk: Int, runtimeGranted: Boolean, enabled: Boolean, asked: Boolean): String =
        if (sdk >= 33) {
            when {
                runtimeGranted && enabled -> "granted"
                !runtimeGranted && !asked -> "notDetermined"
                else -> "denied"
            }
        } else if (enabled) {
            "granted"
        } else {
            "denied"
        }
}

private fun JSONObject.toMap(): Map<String, Any?> =
    keys().asSequence().associateWith { opt(it) }
