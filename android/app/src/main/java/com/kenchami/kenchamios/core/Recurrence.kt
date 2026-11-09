package com.kenchami.kenchamios.core

import java.time.DayOfWeek
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

enum class RepeatUnit(val wire: String) {
    Daily("daily"),
    Weekly("weekly"),
    Monthly("monthly"),
    Weekdays("weekdays"),
}

private val RULE_PATTERN = Regex("^(daily|weekly|monthly|weekdays):([1-9][0-9]{0,2})$")

/** A `tasks.recurrence` value, `unit:interval`; the same rules as the desktop `recurrence.ts`. */
data class RepeatRule(val unit: RepeatUnit, val interval: Int) {
    val wire: String get() = "${unit.wire}:$interval"

    val label: String
        get() {
            val word = when (unit) {
                RepeatUnit.Daily -> "day"
                RepeatUnit.Weekly -> "week"
                RepeatUnit.Monthly -> "month"
                RepeatUnit.Weekdays -> return "Every weekday"
            }
            return if (interval == 1) "Every $word" else "Every $interval ${word}s"
        }

    /**
     * The next due day after completing: steps from [due] (today when there is none) until the result is after [today].
     * Months keep the day of the base date and clamp to the month end; weekdays skip the weekend.
     */
    fun nextDue(due: LocalDate?, today: LocalDate): LocalDate {
        val base = due ?: today
        return generateSequence(1L) { it + 1 }
            .map { step(base, it) }
            .first { it.isAfter(today) && !(unit == RepeatUnit.Weekdays && isWeekend(it)) }
    }

    // months count from the base, so a 31st stays a 31st where the month allows it
    private fun step(base: LocalDate, k: Long): LocalDate = when (unit) {
        RepeatUnit.Daily -> base.plusDays(k * interval)
        RepeatUnit.Weekly -> base.plusWeeks(k * interval)
        RepeatUnit.Monthly -> base.plusMonths(k * interval)
        RepeatUnit.Weekdays -> base.plusDays(k)
    }

    companion object {
        fun parse(text: String): RepeatRule? {
            val match = RULE_PATTERN.matchEntire(text) ?: return null
            val unit = RepeatUnit.entries.first { it.wire == match.groupValues[1] }
            val interval = match.groupValues[2].toInt()
            return if (unit == RepeatUnit.Weekdays && interval != 1) null else RepeatRule(unit, interval)
        }
    }
}

fun isWeekend(day: LocalDate): Boolean = day.dayOfWeek == DayOfWeek.SATURDAY || day.dayOfWeek == DayOfWeek.SUNDAY

/** Local midnight of the next occurrence of a task with [recurrence] due at [dueAt], or null for a one-off task. */
fun nextDueAt(recurrence: String?, dueAt: Long?, now: Long, zone: ZoneId): Long? {
    val rule = recurrence?.let { RepeatRule.parse(it) } ?: return null
    val due = dueAt?.let { Instant.ofEpochMilli(it).atZone(zone).toLocalDate() }
    val today = Instant.ofEpochMilli(now).atZone(zone).toLocalDate()
    return startOfDay(rule.nextDue(due, today), zone)
}
