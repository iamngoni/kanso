package app.kanso.kanso

import java.time.ZoneId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ReminderRulesTest {
    private fun reminder(key: String = "k", day: Int = 25) =
        Reminder(key, "n", 3, "Call Sam", "Today 09:00", 2026, 10, day, 9, 0)

    @Test
    fun remindersSurviveTheirKeptForm() {
        val list = listOf(reminder("a"), reminder("b", day = 26))
        assertEquals(list, ReminderRules.decodeReminders(ReminderRules.encodeReminders(list)))
    }

    @Test
    fun unreadableKeptListsAreEmpty() {
        assertEquals(emptyList<Reminder>(), ReminderRules.decodeReminders(null))
        assertEquals(emptyList<Reminder>(), ReminderRules.decodeReminders("not json"))
        assertEquals(emptyList<Answer>(), ReminderRules.decodeAnswers("{"))
    }

    @Test
    fun aMalformedEntryIsLeftOut() {
        val text = """[{"key":"a"},{"key":"b","noteId":"n","line":1,"year":2026,"month":10,
            "day":25,"hour":9,"minute":0},{"key":"c","noteId":"n","line":1,"year":2026,
            "month":13,"day":1,"hour":9,"minute":0}]"""
        assertEquals(listOf("b"), ReminderRules.decodeReminders(text).map { it.key })
    }

    @Test
    fun answersSurviveTheirKeptForm() {
        val answers = listOf(Answer("a", "n", 2, "mark-done"), Answer("b", "m", 0, "open"))
        assertEquals(answers, ReminderRules.decodeAnswers(ReminderRules.encodeAnswers(answers)))
    }

    @Test
    fun anAnswerCarriesOnlyTheFourFields() {
        assertEquals(
            setOf("key", "noteId", "line", "action"),
            Answer("a", "n", 2, "open").toMap().keys,
        )
    }

    @Test
    fun theReminderMapOfTheChannelParses() {
        val map = mapOf(
            "key" to "k", "noteId" to "n", "line" to 3, "title" to "T", "body" to "B",
            "year" to 2026, "month" to 10, "day" to 25, "hour" to 9, "minute" to 30L,
        )
        assertEquals(30, Reminder.fromMap(map)?.minute)
        assertNull(Reminder.fromMap(map - "noteId"))
    }

    @Test
    fun theIdIsTheSameForTheSameKey() {
        assertEquals(ReminderRules.notificationId("abc"), ReminderRules.notificationId("abc"))
        assertEquals("abc".hashCode(), ReminderRules.notificationId("abc"))
        assertFalse(ReminderRules.requestCode("fire", "abc") == ReminderRules.requestCode("open", "abc"))
    }

    @Test
    fun theFireTimeIsLocal() {
        val at = reminder().triggerAtMillis(ZoneId.of("UTC"))
        val east = reminder().triggerAtMillis(ZoneId.of("Africa/Harare"))
        assertEquals(2 * 60 * 60 * 1000L, at - east)
    }

    @Test
    fun onlyTheUpcomingAreKeptSoonestFirst() {
        val a = reminder("a", day = 27)
        val b = reminder("b", day = 26)
        val gone = reminder("c", day = 1)
        val now = reminder(day = 25).triggerAtMillis()
        assertEquals(listOf("b", "a"), ReminderRules.upcoming(listOf(a, gone, b), now).map { it.key })
    }

    @Test
    fun anAnswerIsLiveOrKeptNeverBoth() {
        assertFalse(ReminderRules.deliverLive("open", engineUp = false, appInFront = true))
        assertTrue(ReminderRules.deliverLive("open", engineUp = true, appInFront = false))
        assertFalse(ReminderRules.deliverLive("mark-done", engineUp = true, appInFront = false))
        assertTrue(ReminderRules.deliverLive("dismiss", engineUp = true, appInFront = true))
    }

    @Test
    fun permissionStates() {
        assertEquals("granted", ReminderRules.permission(34, true, true, true))
        assertEquals("notDetermined", ReminderRules.permission(34, false, false, false))
        assertEquals("denied", ReminderRules.permission(34, false, false, true))
        assertEquals("denied", ReminderRules.permission(34, true, false, true))
        assertEquals("granted", ReminderRules.permission(30, false, true, false))
        assertEquals("denied", ReminderRules.permission(30, false, false, false))
    }
}
