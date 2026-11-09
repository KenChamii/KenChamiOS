package com.kenchami.kenchamios.core

import com.kenchami.kenchamios.TestClock
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.UUID

class Uuid7Test {
    private val clock = TestClock(ms = 1_700_000_000_000)
    private val ids = Uuid7Generator(clock)

    @Test
    fun hasVersionSevenAndTheRfcVariant() {
        val id = UUID.fromString(ids.next())
        assertEquals(7, id.version())
        assertEquals(2, id.variant())
    }

    @Test
    fun startsWithTheUnixMilliseconds() {
        val id = UUID.fromString(ids.next())
        assertEquals(1_700_000_000_000, id.mostSignificantBits ushr 16)
    }

    @Test
    fun sortsInCreationOrderInsideOneMillisecond() {
        val made = List(10_000) { ids.next() }
        assertEquals(made.size, made.toSet().size)
        assertEquals(made, made.sorted())
    }

    @Test
    fun laterTimeSortsAfter() {
        val first = ids.next()
        clock.ms += 1
        assertTrue(ids.next() > first)
    }

    @Test
    fun staysOrderedWhenTheClockGoesBackwards() {
        val first = ids.next()
        clock.ms -= 5_000
        assertTrue(ids.next() > first)
    }
}
