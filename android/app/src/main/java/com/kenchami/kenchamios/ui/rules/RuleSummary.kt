package com.kenchami.kenchamios.ui.rules

import com.kenchami.kenchamios.context.PlaceTransition
import com.kenchami.kenchamios.context.Rule
import com.kenchami.kenchamios.context.RuleAction
import com.kenchami.kenchamios.context.Trigger
import com.kenchami.kenchamios.context.WifiTransition
import java.time.DayOfWeek
import java.time.format.TextStyle
import java.util.Locale

/** One line about when the rule fires, without the coordinates. */
fun triggerSummary(trigger: Trigger): String = when (trigger) {
    is Trigger.Place -> {
        val verb = if (trigger.transition == PlaceTransition.Leave) "leave" else "arrive at"
        "When you $verb a place (${trigger.radiusM} m)"
    }
    is Trigger.Time -> "${daysLabel(trigger.days)} at ${clockLabel(trigger.minuteOfDay)}"
    is Trigger.Wifi -> {
        val verb = if (trigger.transition == WifiTransition.Connect) "join" else "leave"
        "When you $verb a Wi-Fi network"
    }
}

fun actionSummary(action: RuleAction): String = when (action) {
    RuleAction.ShowTasks -> "Show tasks due today"
    is RuleAction.Remind -> "Remind: ${action.text}"
    is RuleAction.Checklist -> "Checklist of ${action.items.size} ${if (action.items.size == 1) "item" else "items"}"
}

fun ruleSummary(rule: Rule): String = "${triggerSummary(rule.trigger)} - ${actionSummary(rule.action)}"

fun clockLabel(minuteOfDay: Int): String = "%02d:%02d".format(minuteOfDay / MINUTES_PER_HOUR, minuteOfDay % MINUTES_PER_HOUR)

fun dayLabel(day: DayOfWeek): String = day.getDisplayName(TextStyle.SHORT, Locale.ENGLISH)

private fun daysLabel(days: Set<DayOfWeek>): String = when (days.size) {
    DayOfWeek.entries.size -> "Every day"
    else -> days.sorted().joinToString(", ", transform = ::dayLabel)
}

private const val MINUTES_PER_HOUR = 60
