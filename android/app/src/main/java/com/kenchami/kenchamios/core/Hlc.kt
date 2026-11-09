package com.kenchami.kenchamios.core

import java.time.Clock
import java.util.concurrent.atomic.AtomicLong

// low bits count events inside the same millisecond
private const val COUNTER_BITS = 16

/** Hybrid logical clock: unix ms shifted left by 16 bits, so values stay ordered by time and never repeat. */
class Hlc(private val clock: Clock) {
    private val last = AtomicLong(0)

    fun now(): Long = advance(clock.millis() shl COUNTER_BITS)

    /** Call with every remote value so local values always sort after it. */
    fun merge(remote: Long): Long = advance(maxOf(clock.millis() shl COUNTER_BITS, saturatingInc(remote)))

    // hlc must never go backwards, even if the system clock does
    private fun advance(floor: Long): Long = last.updateAndGet { maxOf(saturatingInc(it), floor) }

    private fun saturatingInc(value: Long): Long = if (value == Long.MAX_VALUE) value else value + 1
}
