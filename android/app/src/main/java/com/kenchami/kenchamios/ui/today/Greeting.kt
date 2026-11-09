package com.kenchami.kenchamios.ui.today

import java.time.LocalDate
import java.time.LocalTime
import java.time.format.DateTimeFormatter
import java.util.Locale

private const val MORNING_FROM = 5
private const val AFTERNOON_FROM = 12
private const val EVENING_FROM = 18

fun greeting(time: LocalTime): String = when (time.hour) {
    in MORNING_FROM until AFTERNOON_FROM -> "Good morning"
    in AFTERNOON_FROM until EVENING_FROM -> "Good afternoon"
    else -> "Good evening"
}

/** "Tuesday, 10 November" */
fun dateLabel(date: LocalDate, locale: Locale = Locale.getDefault()): String =
    DateTimeFormatter.ofPattern("EEEE, d MMMM", locale).format(date)
