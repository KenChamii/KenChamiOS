package com.kenchami.kenchamios.context

import java.time.DayOfWeek
import java.time.Instant
import java.time.ZoneId

private const val LOOKUP_DAYS = 8L
private const val MINUTES_PER_HOUR = 60

/** When a time rule is due. Local times that do not exist (spring DST) move forward, repeated ones use the first. */
object RuleSchedule {
    /** The first scheduled instant after [after], or null when no weekday is chosen. */
    fun next(after: Long, minuteOfDay: Int, days: Set<DayOfWeek>, zone: ZoneId): Long? =
        instants(after, minuteOfDay, days, zone, -1L..LOOKUP_DAYS).firstOrNull { it > after }

    /** The latest scheduled instant at or before [now], or null when no weekday is chosen. */
    fun previous(now: Long, minuteOfDay: Int, days: Set<DayOfWeek>, zone: ZoneId): Long? =
        instants(now, minuteOfDay, days, zone, -LOOKUP_DAYS..1L).lastOrNull { it <= now }

    private fun instants(around: Long, minuteOfDay: Int, days: Set<DayOfWeek>, zone: ZoneId, offsets: LongRange): Sequence<Long> {
        val today = Instant.ofEpochMilli(around).atZone(zone).toLocalDate()
        return offsets.asSequence()
            .map { today.plusDays(it) }
            .filter { it.dayOfWeek in days }
            .map { it.atTime(minuteOfDay / MINUTES_PER_HOUR, minuteOfDay % MINUTES_PER_HOUR).atZone(zone).toInstant().toEpochMilli() }
            .sorted()
    }
}
