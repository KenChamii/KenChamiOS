package com.kenchami.kenchamios.core

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import java.time.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

/** Local midnight that starts the day, and the one that ends it, in unix ms. */
data class DayBounds(val start: Long, val end: Long)

fun dayBounds(now: Long, zone: ZoneId): DayBounds {
    val date = Instant.ofEpochMilli(now).atZone(zone).toLocalDate()
    return DayBounds(start = startOfDay(date, zone), end = startOfDay(date.plusDays(1), zone))
}

/** Local midnight of [date], the value the desktop stores as a due date. */
fun startOfDay(date: LocalDate, zone: ZoneId): Long = date.atStartOfDay(zone).toInstant().toEpochMilli()

/** Today's local date, again at each midnight while it is collected. */
fun localDates(clock: Clock): Flow<LocalDate> = flow {
    while (true) {
        val today = LocalDate.now(clock)
        emit(today)
        delay(startOfDay(today.plusDays(1), clock.zone) - clock.millis())
    }
}
