package com.kenchami.kenchamios.context

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.time.DayOfWeek
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZoneOffset

class RuleScheduleTest {
    private val berlin = ZoneId.of("Europe/Berlin")
    private val everyDay = DayOfWeek.entries.toSet()
    private val weekdays = setOf(DayOfWeek.MONDAY, DayOfWeek.TUESDAY, DayOfWeek.WEDNESDAY, DayOfWeek.THURSDAY, DayOfWeek.FRIDAY)

    private fun ms(local: String, zone: ZoneId = berlin) = LocalDateTime.parse(local).atZone(zone).toInstant().toEpochMilli()

    @Test
    fun nextIsLaterTodayWhenTheTimeIsStillAhead() {
        assertEquals(ms("2026-06-10T08:00"), RuleSchedule.next(ms("2026-06-10T07:00"), 8 * 60, everyDay, berlin))
    }

    @Test
    fun nextIsTomorrowWhenTheTimeHasPassedOrIsNow() {
        val expected = ms("2026-06-11T08:00")
        assertEquals(expected, RuleSchedule.next(ms("2026-06-10T09:00"), 8 * 60, everyDay, berlin))
        assertEquals(expected, RuleSchedule.next(ms("2026-06-10T08:00"), 8 * 60, everyDay, berlin))
    }

    @Test
    fun nextSkipsTheDaysThatAreNotChosen() {
        // 2026-06-12 is a Friday, the next weekday slot is Monday
        assertEquals(ms("2026-06-15T08:00"), RuleSchedule.next(ms("2026-06-12T09:00"), 8 * 60, weekdays, berlin))
        assertEquals(ms("2026-06-13T08:00"), RuleSchedule.next(ms("2026-06-12T09:00"), 8 * 60, setOf(DayOfWeek.SATURDAY), berlin))
    }

    @Test
    fun nextAndPreviousAreNullWithoutDays() {
        assertNull(RuleSchedule.next(0, 60, emptySet(), berlin))
        assertNull(RuleSchedule.previous(0, 60, emptySet(), berlin))
    }

    @Test
    fun previousIsTheLatestSlotAtOrBeforeNow() {
        assertEquals(ms("2026-06-10T08:00"), RuleSchedule.previous(ms("2026-06-10T09:00"), 8 * 60, everyDay, berlin))
        assertEquals(ms("2026-06-10T08:00"), RuleSchedule.previous(ms("2026-06-10T08:00"), 8 * 60, everyDay, berlin))
        assertEquals(ms("2026-06-09T08:00"), RuleSchedule.previous(ms("2026-06-10T07:59"), 8 * 60, everyDay, berlin))
        assertEquals(ms("2026-06-12T08:00"), RuleSchedule.previous(ms("2026-06-14T12:00"), 8 * 60, weekdays, berlin))
    }

    @Test
    fun theTimeFollowsTheZoneNotTheOffset() {
        val newYork = ZoneId.of("America/New_York")
        assertEquals(ms("2026-06-10T08:00", newYork), RuleSchedule.next(ms("2026-06-10T07:00", newYork), 8 * 60, everyDay, newYork))
    }

    @Test
    fun aTimeThatDoesNotExistMovesForwardOnTheSpringChangeDay() {
        // Berlin skips 02:00 to 03:00 on 2026-03-29
        val next = RuleSchedule.next(ms("2026-03-28T12:00"), 2 * 60 + 30, everyDay, berlin)
        assertEquals(ms("2026-03-29T03:30"), next)
    }

    @Test
    fun aRepeatedTimeFiresOnceOnTheAutumnChangeDay() {
        // Berlin repeats 02:00 to 03:00 on 2026-10-25: the first 02:30 is at 00:30 UTC
        val first = RuleSchedule.next(ms("2026-10-24T12:00"), 2 * 60 + 30, everyDay, berlin)
        assertEquals(LocalDateTime.parse("2026-10-25T00:30").toInstant(ZoneOffset.UTC).toEpochMilli(), first)
        val following = RuleSchedule.next(first ?: 0, 2 * 60 + 30, everyDay, berlin)
        assertEquals(ms("2026-10-26T02:30"), following)
    }

    @Test
    fun theDayIsDecidedInTheRuleZone() {
        // 23:30 UTC on Friday is already Saturday in Berlin
        val now = LocalDateTime.parse("2026-06-12T23:30").toInstant(ZoneOffset.UTC).toEpochMilli()
        assertEquals(ms("2026-06-13T08:00"), RuleSchedule.next(now, 8 * 60, setOf(DayOfWeek.SATURDAY), berlin))
    }
}
