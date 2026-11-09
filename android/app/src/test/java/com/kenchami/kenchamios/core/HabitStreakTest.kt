package com.kenchami.kenchamios.core

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.LocalDate

// the same cases as the desktop domain/habit.rs
class HabitStreakTest {
    private fun summary(days: List<String>, today: String) = summarizeHabit("h", "Read", days, LocalDate.parse(today))

    @Test
    fun countsTheRunThatEndsToday() {
        val habit = summary(listOf("2026-11-08", "2026-11-09", "2026-11-10"), "2026-11-10")

        assertEquals(3, habit.streak)
        assertEquals(true, habit.checkedToday)
        assertEquals(listOf(false, false, false, false, true, true, true), habit.recent)
    }

    @Test
    fun todayNotYetCheckedKeepsYesterdaysStreak() {
        val habit = summary(listOf("2026-11-08", "2026-11-09"), "2026-11-10")

        assertEquals(2, habit.streak)
        assertEquals(false, habit.checkedToday)
    }

    @Test
    fun aMissedDayBreaksTheStreak() {
        assertEquals(0, summary(listOf("2026-11-07", "2026-11-08"), "2026-11-10").streak)
        assertEquals(3, summary(listOf("2026-11-06", "2026-11-08", "2026-11-09", "2026-11-10"), "2026-11-10").streak)
    }

    @Test
    fun streaksRunAcrossMonthAndYearEnds() {
        assertEquals(3, summary(listOf("2026-12-30", "2026-12-31", "2027-01-01"), "2027-01-01").streak)
        assertEquals(3, summary(listOf("2024-02-28", "2024-02-29", "2024-03-01"), "2024-03-01").streak)
    }

    @Test
    fun noChecksAndUnreadableDays() {
        assertEquals(0, summary(emptyList(), "2026-11-10").streak)
        assertEquals(1, summary(listOf("garbage", "2026-02-30", "2026-11-10"), "2026-11-10").streak)
    }
}
