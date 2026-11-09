package com.kenchami.kenchamios.ui.common

import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

private const val MINUTE = 60_000L
private const val HOUR = 60 * MINUTE
private const val DAY = 24 * HOUR

fun timeAgo(ms: Long, now: Long): String {
    val elapsed = maxOf(0, now - ms)
    return when {
        elapsed < MINUTE -> "just now"
        elapsed < HOUR -> "${elapsed / MINUTE}m ago"
        elapsed < DAY -> "${elapsed / HOUR}h ago"
        else -> "${elapsed / DAY}d ago"
    }
}

fun formatDate(ms: Long, zone: ZoneId): String =
    DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM).format(Instant.ofEpochMilli(ms).atZone(zone))

/** m:ss, or h:mm:ss from an hour on. */
fun formatClock(totalSeconds: Int): String {
    val hours = totalSeconds / 3600
    val minutes = totalSeconds % 3600 / 60
    val seconds = totalSeconds % 60
    return if (hours > 0) "%d:%02d:%02d".format(hours, minutes, seconds) else "%d:%02d".format(minutes, seconds)
}
