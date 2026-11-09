package com.kenchami.kenchamios.context

const val MAX_PLACE_RULES = 100
const val MIN_RADIUS_M = 100
const val MAX_RADIUS_M = 5000
const val MAX_NAME = 60
const val MAX_TEXT = 200
const val MAX_ITEMS = 20
const val MAX_ITEM = 60

private const val MAX_LATITUDE = 90.0
private const val MAX_LONGITUDE = 180.0
private const val LAST_MINUTE = 24 * 60 - 1

/** The first problem of a rule as a sentence for the editor, or null when the rule can be saved. */
object RuleValidation {
    /** [otherEnabledPlaceRules] counts the enabled place rules besides this one; the platform allows 100 geofences. */
    fun check(rule: Rule, otherEnabledPlaceRules: Int): String? {
        if (rule.name.isBlank()) return "Give the rule a name."
        if (rule.name.length > MAX_NAME) return "The name is longer than $MAX_NAME characters."
        return checkTrigger(rule, otherEnabledPlaceRules) ?: checkAction(rule.action)
    }

    private fun checkTrigger(rule: Rule, otherEnabledPlaceRules: Int): String? = when (val trigger = rule.trigger) {
        is Trigger.Place -> when {
            trigger.latitude !in -MAX_LATITUDE..MAX_LATITUDE -> "Latitude must be between -90 and 90."
            trigger.longitude !in -MAX_LONGITUDE..MAX_LONGITUDE -> "Longitude must be between -180 and 180."
            trigger.radiusM !in MIN_RADIUS_M..MAX_RADIUS_M -> "The radius must be between $MIN_RADIUS_M and $MAX_RADIUS_M metres."
            rule.enabled && otherEnabledPlaceRules >= MAX_PLACE_RULES ->
                "Android allows $MAX_PLACE_RULES place rules at most. Turn one off."
            else -> null
        }
        is Trigger.Time -> when {
            trigger.minuteOfDay !in 0..LAST_MINUTE -> "Pick a time of day."
            trigger.days.isEmpty() -> "Pick at least one weekday."
            else -> null
        }
        is Trigger.Wifi -> null
    }

    private fun checkAction(action: RuleAction): String? = when (action) {
        RuleAction.ShowTasks -> null
        is RuleAction.Remind -> when {
            action.text.isBlank() -> "Write the reminder text."
            action.text.length > MAX_TEXT -> "The text is longer than $MAX_TEXT characters."
            else -> null
        }
        is RuleAction.Checklist -> when {
            action.items.isEmpty() -> "Add at least one checklist item."
            action.items.size > MAX_ITEMS -> "A checklist can have $MAX_ITEMS items at most."
            action.items.any { it.isBlank() } -> "Checklist items cannot be empty."
            action.items.any { it.length > MAX_ITEM } -> "An item is longer than $MAX_ITEM characters."
            else -> null
        }
    }
}
