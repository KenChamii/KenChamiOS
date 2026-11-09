package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.ui.common.formatClock
import com.kenchami.kenchamios.ui.common.timeAgo
import org.junit.Assert.assertEquals
import org.junit.Test

class FormatTest {
    @Test
    fun timeAgoUsesTheLargestUnit() {
        val now = 10 * 86_400_000L
        assertEquals("just now", timeAgo(now - 59_000, now))
        assertEquals("5m ago", timeAgo(now - 5 * 60_000, now))
        assertEquals("3h ago", timeAgo(now - 3 * 3_600_000, now))
        assertEquals("2d ago", timeAgo(now - 2 * 86_400_000, now))
    }

    @Test
    fun timeAgoTreatsTheFutureAsNow() {
        assertEquals("just now", timeAgo(5_000, 1_000))
    }

    @Test
    fun clockShowsMinutesAndSecondsAndHoursFromAnHour() {
        assertEquals("25:00", formatClock(25 * 60))
        assertEquals("0:05", formatClock(5))
        assertEquals("1:01:01", formatClock(3_661))
    }
}
