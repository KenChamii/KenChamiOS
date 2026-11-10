package com.kenchami.kenchamios.core

import java.time.LocalDate
import java.time.format.DateTimeParseException

const val RECENT_DAYS = 7

/** One habit as the list shows it; the same numbers as the desktop `domain/habit.rs`. */
data class HabitSummary(
    val id: String,
    val name: String,
    /** days in a row up to today, or up to yesterday while today is still open */
    val streak: Int,
    val checkedToday: Boolean,
    /** the last seven days, oldest first, today last */
    val recent: List<Boolean>,
)

/** [days] are the habit's checked `YYYY-MM-DD` days; unreadable ones are skipped. */
fun summarizeHabit(id: String, name: String, days: List<String>, today: LocalDate): HabitSummary {
    val checked = days.mapNotNull { parseDay(it) }.toSet()
    val checkedToday = today in checked
    var cursor = if (checkedToday) today else today.minusDays(1)
    var streak = 0
    while (cursor in checked) {
        streak++
        cursor = cursor.minusDays(1)
    }
    val recent = (RECENT_DAYS - 1 downTo 0).map { today.minusDays(it.toLong()) in checked }
    return HabitSummary(id, name, streak, checkedToday, recent)
}

private val DAY_PATTERN = Regex("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")

/** A real calendar day written as `YYYY-MM-DD`, or null. */
fun parseDay(text: String): LocalDate? {
    if (!DAY_PATTERN.matches(text)) return null
    return try {
        LocalDate.parse(text)
    } catch (_: DateTimeParseException) {
        null
    }
}
