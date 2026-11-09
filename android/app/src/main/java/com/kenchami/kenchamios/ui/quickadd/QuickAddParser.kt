package com.kenchami.kenchamios.ui.quickadd

import com.kenchami.kenchamios.core.isWeekend
import com.kenchami.kenchamios.core.parseDay
import com.kenchami.kenchamios.data.model.Priority
import java.time.DayOfWeek
import java.time.LocalDate

private val WEEKDAYS = listOf("monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday")

private val PRIORITY_WORDS = mapOf(
    "!high" to Priority.High,
    "!h" to Priority.High,
    "!medium" to Priority.Medium,
    "!med" to Priority.Medium,
    "!m" to Priority.Medium,
    "!low" to Priority.Low,
    "!l" to Priority.Low,
)

private val SINGLE_REPEATS = mapOf("daily" to "daily:1", "weekly" to "weekly:1", "monthly" to "monthly:1", "weekdays" to "weekdays:1")

private val EVERY_UNITS = mapOf(
    "day" to "daily",
    "days" to "daily",
    "week" to "weekly",
    "weeks" to "weekly",
    "month" to "monthly",
    "months" to "monthly",
)

private val COUNT_PATTERN = Regex("^[1-9][0-9]{0,2}$")
private val PROJECT_PATTERN = Regex("""^#[\p{L}\p{N}_-]+$""")
private val SPACES = Regex("""\s+""")

private fun weekdayOf(word: String?): DayOfWeek? = WEEKDAYS.indexOf(word).takeIf { it >= 0 }?.let { DayOfWeek.of(it + 1) }

// the next date with this weekday, strictly after today, or from today on when orToday
private fun nextWeekday(today: LocalDate, weekday: DayOfWeek, orToday: Boolean): LocalDate {
    val ahead = (weekday.value - today.dayOfWeek.value + 7) % 7
    return today.plusDays(if (ahead == 0 && !orToday) 7L else ahead.toLong())
}

private fun count(word: String?): Int? = word?.takeIf { COUNT_PATTERN.matches(it) }?.toInt()

private class Match(val value: String, val due: LocalDate?, val length: Int)

// "daily", "every 3 days", "every weekday", "every monday"
private fun readRepeat(words: List<String>, at: Int, today: LocalDate): Match? {
    val word = words[at]
    SINGLE_REPEATS[word]?.let { return Match(it, null, 1) }
    if (word != "every") return null
    val next = words.getOrNull(at + 1)
    if (next == "weekday") return Match("weekdays:1", null, 2)
    EVERY_UNITS[next]?.let { return Match("$it:1", null, 2) }
    weekdayOf(next)?.let { return Match("weekly:1", nextWeekday(today, it, orToday = true), 2) }
    val interval = count(next)
    val unit = EVERY_UNITS[words.getOrNull(at + 2)]
    return if (interval != null && unit != null) Match("$unit:$interval", null, 3) else null
}

// "today", "tmr", "friday", "on friday", "next friday", "next week", "in 3 days", "2026-12-01"
private fun readDue(words: List<String>, original: List<String>, at: Int, today: LocalDate): Pair<LocalDate, Int>? {
    val word = words[at]
    if (word == "today") return today to 1
    if (word == "tomorrow" || word == "tmr") return today.plusDays(1) to 1
    parseDay(original[at])?.let { return it to 1 }
    val lead = if (word == "on" || word == "next") 1 else 0
    weekdayOf(words.getOrNull(at + lead))?.let { return nextWeekday(today, it, orToday = false) to lead + 1 }
    if (word == "next" && words.getOrNull(at + 1) == "week") return nextWeekday(today, DayOfWeek.MONDAY, orToday = false) to 2
    val amount = count(words.getOrNull(at + 1))
    if (word != "in" || amount == null) return null
    return when (words.getOrNull(at + 2)) {
        "day", "days" -> today.plusDays(amount.toLong()) to 3
        "week", "weeks" -> today.plusWeeks(amount.toLong()) to 3
        else -> null
    }
}

/**
 * Reads dates, `!priority`, `#project` and repeats out of a quick-add line, left to right;
 * the first match of each kind wins and the rest stays in the title.
 * Cases shared with the desktop: `shared/fixtures/quick-add/parse.json`.
 */
fun parseQuickAdd(input: String, today: LocalDate): QuickTask {
    val original = input.trim().split(SPACES).filter { it.isNotEmpty() }
    val words = original.map { it.lowercase() }
    val title = mutableListOf<String>()
    var due: LocalDate? = null
    var repeatDue: LocalDate? = null
    var priority: Priority? = null
    var project: String? = null
    var recurrence: String? = null

    var at = 0
    while (at < words.size) {
        val repeat = if (recurrence == null) readRepeat(words, at, today) else null
        if (repeat != null) {
            recurrence = repeat.value
            repeatDue = repeat.due
            at += repeat.length
            continue
        }
        val dueMatch = if (due == null) readDue(words, original, at, today) else null
        if (dueMatch != null) {
            due = dueMatch.first
            at += dueMatch.second
            continue
        }
        val wordPriority = PRIORITY_WORDS[words[at]]
        when {
            priority == null && wordPriority != null -> priority = wordPriority
            project == null && PROJECT_PATTERN.matches(words[at]) -> project = original[at].substring(1)
            else -> title += original[at]
        }
        at++
    }

    if (due == null) due = repeatDue
    if (due == null && recurrence != null) {
        due = if (recurrence == "weekdays:1" && isWeekend(today)) nextWeekday(today, DayOfWeek.MONDAY, orToday = false) else today
    }
    return QuickTask(title.joinToString(" "), due, priority, project, recurrence)
}
