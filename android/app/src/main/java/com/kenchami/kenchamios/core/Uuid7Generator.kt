package com.kenchami.kenchamios.core

import java.security.SecureRandom
import java.time.Clock
import java.util.UUID

private const val COUNTER_LIMIT = 0x1000

/** UUID v7 ids: 48 bit unix ms, then a counter, so ids made in one millisecond still sort in order. */
class Uuid7Generator(private val clock: Clock, private val random: SecureRandom = SecureRandom()) {
    private var lastMs = 0L
    private var counter = 0

    @Synchronized
    fun next(): String {
        var ms = clock.millis()
        if (ms > lastMs) {
            counter = random.nextInt(COUNTER_LIMIT / 2)
        } else {
            ms = lastMs
            counter++
            if (counter == COUNTER_LIMIT) {
                ms++
                counter = 0
            }
        }
        lastMs = ms
        val high = (ms shl 16) or 0x7000L or counter.toLong()
        val low = (random.nextLong() and 0x3FFFFFFFFFFFFFFFL) or Long.MIN_VALUE
        return UUID(high, low).toString()
    }
}
