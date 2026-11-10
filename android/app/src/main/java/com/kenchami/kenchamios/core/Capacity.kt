package com.kenchami.kenchamios.core

import kotlin.math.roundToInt

/** Estimates offered when planning, in minutes; the same list as the desktop. */
val ESTIMATES = listOf(15, 30, 45, 60, 90, 120, 180, 240)
const val DEFAULT_ESTIMATE_MIN = 30
const val DEFAULT_AVAILABLE_MIN = 6 * 60
const val MIN_AVAILABLE_MIN = 30
const val MAX_DURATION_MIN = 24 * 60

private const val MINUTES_PER_HOUR = 60
private val HOURS = Regex("^\\d+(\\.\\d+)?$")

/** The day's planned minutes against the minutes available. */
data class Capacity(
    val plannedMin: Int,
    val availableMin: Int,
    /** minutes planned beyond what is available, 0 when it fits */
    val overMin: Int,
    /** tasks without an estimate, they count as 0 */
    val unestimated: Int,
    /** planned share of the available time, capped at 1 for the bar */
    val fill: Float,
)

fun capacity(estimates: List<Int?>, availableMin: Int): Capacity {
    val planned = estimates.sumOf { it ?: 0 }
    return Capacity(
        plannedMin = planned,
        availableMin = availableMin,
        overMin = maxOf(0, planned - availableMin),
        unestimated = estimates.count { it == null },
        fill = if (availableMin > 0) minOf(1f, planned.toFloat() / availableMin) else 1f,
    )
}

fun isValidAvailableMinutes(minutes: Int) = minutes in MIN_AVAILABLE_MIN..MAX_DURATION_MIN

/** Hours typed in settings ("6", "7.5") as minutes, or null when out of range. */
fun parseHours(text: String): Int? {
    val trimmed = text.trim()
    if (!HOURS.matches(trimmed)) return null
    val minutes = (trimmed.toDouble() * MINUTES_PER_HOUR).roundToInt()
    return minutes.takeIf { isValidAvailableMinutes(it) }
}

/** Minutes as "2h 05m" or "45m", like the desktop. */
fun formatMinutes(minutes: Int): String {
    val hours = minutes / MINUTES_PER_HOUR
    val rest = minutes % MINUTES_PER_HOUR
    return if (hours == 0) "${rest}m" else "${hours}h %02dm".format(rest)
}
