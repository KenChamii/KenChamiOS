package com.kenchami.kenchamios.context

import java.time.ZoneId

const val COOLDOWN_MS = 10 * 60 * 1000L
const val TIME_GRACE_MS = 2 * 60 * 60 * 1000L

/** Decides which rules an event fires; it has no side effects and knows nothing about Android. */
object RuleEngine {
    fun evaluate(event: ContextEvent, rules: List<Rule>, lastFired: Map<String, Long>, now: Long, zone: ZoneId): List<Rule> =
        rules.filter { it.enabled && matches(it, event, lastFired[it.id], now, zone) }

    private fun matches(rule: Rule, event: ContextEvent, lastFired: Long?, now: Long, zone: ZoneId): Boolean {
        val trigger = rule.trigger
        return when {
            event is ContextEvent.PlaceChanged && trigger is Trigger.Place ->
                event.ruleId == rule.id && event.transition == trigger.transition && !coolingDown(lastFired, now)
            event is ContextEvent.WifiChanged && trigger is Trigger.Wifi ->
                event.transition == trigger.transition && !coolingDown(lastFired, now)
            event is ContextEvent.TimeCheck && trigger is Trigger.Time -> dueTime(trigger, lastFired, now, zone)
            else -> false
        }
    }

    private fun coolingDown(lastFired: Long?, now: Long) = lastFired != null && now - lastFired < COOLDOWN_MS

    private fun dueTime(trigger: Trigger.Time, lastFired: Long?, now: Long, zone: ZoneId): Boolean {
        val scheduled = RuleSchedule.previous(now, trigger.minuteOfDay, trigger.days, zone) ?: return false
        return scheduled > (lastFired ?: Long.MIN_VALUE) && now - scheduled <= TIME_GRACE_MS
    }
}
