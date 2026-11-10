package com.kenchami.kenchamios.core

import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.io.File
import java.time.LocalDate
import java.time.ZoneOffset

// org.json is part of Android, so the fixture tests run on Robolectric
@RunWith(RobolectricTestRunner::class)
class RecurrenceTest {
    private val fixture = JSONObject(File("../../shared/fixtures/recurrence/rules.json").readText())

    private fun strings(array: JSONArray) = List(array.length()) { array.getString(it) }

    @Test
    fun sharedValidRulesParseAndRoundTrip() {
        for (text in strings(fixture.getJSONArray("valid"))) {
            assertEquals(text, RepeatRule.parse(text)?.wire)
        }
    }

    @Test
    fun sharedInvalidRulesAreRefused() {
        for (text in strings(fixture.getJSONArray("invalid"))) {
            assertNull("'$text'", RepeatRule.parse(text))
        }
    }

    @Test
    fun sharedNextDueCases() {
        val cases = fixture.getJSONArray("next")
        for (index in 0 until cases.length()) {
            val case = cases.getJSONObject(index)
            val rule = checkNotNull(RepeatRule.parse(case.getString("rule")))
            val due = if (case.isNull("due")) null else LocalDate.parse(case.getString("due"))

            val next = rule.nextDue(due, LocalDate.parse(case.getString("today")))

            assertEquals(case.toString(), case.getString("expected"), next.toString())
        }
    }

    @Test
    fun nextDueAtIsLocalMidnightAndNullForOneOffTasks() {
        val zone = ZoneOffset.ofHours(8)
        val now = LocalDate.of(2026, 11, 10).atTime(23, 30).atZone(zone).toInstant().toEpochMilli()

        assertEquals(startOfDay(LocalDate.of(2026, 11, 11), zone), nextDueAt("daily:1", null, now, zone))
        assertNull(nextDueAt(null, null, now, zone))
        assertNull(nextDueAt("yearly:1", null, now, zone))
    }

    @Test
    fun labelsReadLikeTheDesktop() {
        assertEquals("Every week", RepeatRule.parse("weekly:1")?.label)
        assertEquals("Every 3 days", RepeatRule.parse("daily:3")?.label)
        assertEquals("Every weekday", RepeatRule.parse("weekdays:1")?.label)
    }
}
