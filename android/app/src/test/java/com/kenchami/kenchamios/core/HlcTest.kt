package com.kenchami.kenchamios.core

import com.kenchami.kenchamios.TestClock
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class HlcTest {
    private val clock = TestClock(ms = 1_000)
    private val hlc = Hlc(clock)

    @Test
    fun valueIsTheWallClockShiftedLeftBySixteenBits() {
        assertEquals(1_000L shl 16, hlc.now())
    }

    @Test
    fun increasesOverManyCallsInTheSameMillisecond() {
        var previous = hlc.now()
        repeat(100_000) {
            val next = hlc.now()
            assertTrue(next > previous)
            previous = next
        }
    }

    @Test
    fun keepsIncreasingWhenTheClockGoesBackwards() {
        val ahead = hlc.now()
        clock.ms = 10
        assertTrue(hlc.now() > ahead)
    }

    @Test
    fun mergeSortsAfterTheRemoteValue() {
        val remote = hlc.now() + (5_000L shl 16)
        assertTrue(hlc.merge(remote) > remote)
        assertTrue(hlc.now() > remote)
    }

    @Test
    fun mergeIgnoresAnOlderRemoteValue() {
        val local = hlc.now()
        assertTrue(hlc.merge(1) > local)
    }

    @Test
    fun mergeDoesNotOverflowAtTheLargestValue() {
        assertEquals(Long.MAX_VALUE, hlc.merge(Long.MAX_VALUE))
    }
}
