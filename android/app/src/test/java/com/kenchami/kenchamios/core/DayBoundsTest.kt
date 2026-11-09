package com.kenchami.kenchamios.core

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZoneOffset

class DayBoundsTest {
    @Test
    fun theDayRunsFromLocalMidnightToTheNext() {
        val zone = ZoneOffset.ofHours(2)
        val noon = LocalDate.of(2026, 3, 10).atTime(12, 0).atZone(zone).toInstant().toEpochMilli()

        val bounds = dayBounds(noon, zone)

        assertEquals(LocalDate.of(2026, 3, 10).atStartOfDay(zone).toInstant().toEpochMilli(), bounds.start)
        assertEquals(bounds.start + 24 * 3_600_000L, bounds.end)
    }

    @Test
    fun midnightBelongsToTheNewDay() {
        val zone = ZoneOffset.UTC
        val midnight = startOfDay(LocalDate.of(2026, 3, 10), zone)

        assertEquals(midnight, dayBounds(midnight, zone).start)
        assertEquals(midnight - 24 * 3_600_000L, dayBounds(midnight - 1, zone).start)
    }

    @Test
    fun aDayWithADaylightSavingShiftIsNotTwentyFourHours() {
        val zone = ZoneId.of("Europe/Berlin")
        val shiftDay = LocalDate.of(2026, 3, 29)

        val bounds = dayBounds(startOfDay(shiftDay, zone) + 1, zone)

        assertEquals(23 * 3_600_000L, bounds.end - bounds.start)
    }
}
