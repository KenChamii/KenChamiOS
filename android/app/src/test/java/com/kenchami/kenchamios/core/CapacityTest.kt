package com.kenchami.kenchamios.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class CapacityTest {
    @Test
    fun aPlanThatFitsHasNoOverflow() {
        val fit = capacity(listOf(30, 60, null), 360)

        assertEquals(Capacity(plannedMin = 90, availableMin = 360, overMin = 0, unestimated = 1, fill = 0.25f), fit)
    }

    @Test
    fun aPlanBeyondTheHoursShowsHowMuchAndAFullBar() {
        val over = capacity(listOf(240, 180), 360)

        assertEquals(60, over.overMin)
        assertEquals(1f, over.fill)
        assertEquals(Capacity(0, 360, 0, 0, 0f), capacity(emptyList(), 360))
    }

    @Test
    fun hoursAreParsedWithinHalfAnHourAndADay() {
        assertEquals(360, parseHours("6"))
        assertEquals(450, parseHours(" 7.5 "))
        assertEquals(30, parseHours("0.5"))
        assertEquals(1440, parseHours("24"))
        for (bad in listOf("", "0.4", "24.5", "-1", "six", "6h", "1e1")) assertNull(bad, parseHours(bad))
    }

    @Test
    fun minutesReadLikeTheDesktop() {
        assertEquals("45m", formatMinutes(45))
        assertEquals("2h 05m", formatMinutes(125))
        assertEquals("6h 00m", formatMinutes(360))
        assertEquals("0m", formatMinutes(0))
    }
}
