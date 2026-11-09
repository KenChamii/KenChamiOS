package com.kenchami.kenchamios.ui.rules

import com.kenchami.kenchamios.context.PlaceTransition
import com.kenchami.kenchamios.context.Rule
import com.kenchami.kenchamios.context.RuleAction
import com.kenchami.kenchamios.context.Trigger
import com.kenchami.kenchamios.context.WifiTransition
import java.time.DayOfWeek

enum class TriggerKind(val label: String) { Place("Place"), Time("Time"), Wifi("Wi-Fi") }

enum class ActionKind(val label: String) { ShowTasks("Show tasks"), Remind("Remind"), Checklist("Checklist") }

private const val MINUTES_PER_HOUR = 60
private const val DEFAULT_RADIUS = "150"
private const val DEFAULT_HOUR = "8"
private const val LAST_HOUR = 23
private const val LAST_MINUTE = 59
private const val LEAVING_HOME_ITEMS = "Keys\nWallet\nPhone"
private val WEEKDAYS = setOf(DayOfWeek.MONDAY, DayOfWeek.TUESDAY, DayOfWeek.WEDNESDAY, DayOfWeek.THURSDAY, DayOfWeek.FRIDAY)

/** The editor's fields as typed text, so a half-written number never has to be a valid rule. */
data class RuleForm(
    val name: String = "",
    val enabled: Boolean = true,
    val trigger: TriggerKind = TriggerKind.Place,
    val latitude: String = "",
    val longitude: String = "",
    val radius: String = DEFAULT_RADIUS,
    val placeTransition: PlaceTransition = PlaceTransition.Leave,
    val hour: String = DEFAULT_HOUR,
    val minute: String = "0",
    val days: Set<DayOfWeek> = WEEKDAYS,
    val wifiTransition: WifiTransition = WifiTransition.Connect,
    val action: ActionKind = ActionKind.Remind,
    val text: String = "",
    val items: String = "",
) {
    /** The rule, or the sentence that tells the user what to fix. */
    fun toRule(id: String): FormResult {
        val trigger = when (trigger) {
            TriggerKind.Place -> placeTrigger() ?: return FormResult.Invalid("Latitude, longitude and radius must be numbers.")
            TriggerKind.Time -> timeTrigger() ?: return FormResult.Invalid("Hour is 0 to $LAST_HOUR and minute is 0 to $LAST_MINUTE.")
            TriggerKind.Wifi -> Trigger.Wifi(wifiTransition)
        }
        return FormResult.Valid(Rule(id, name.trim(), enabled, trigger, buildAction()))
    }

    private fun placeTrigger(): Trigger.Place? {
        val lat = latitude.trim().toDoubleOrNull() ?: return null
        val lon = longitude.trim().toDoubleOrNull() ?: return null
        val metres = radius.trim().toIntOrNull() ?: return null
        return Trigger.Place(lat, lon, metres, placeTransition)
    }

    private fun timeTrigger(): Trigger.Time? {
        val h = hour.trim().toIntOrNull()?.takeIf { it in 0..LAST_HOUR } ?: return null
        val m = minute.trim().toIntOrNull()?.takeIf { it in 0..LAST_MINUTE } ?: return null
        return Trigger.Time(h * MINUTES_PER_HOUR + m, days)
    }

    private fun buildAction(): RuleAction = when (action) {
        ActionKind.ShowTasks -> RuleAction.ShowTasks
        ActionKind.Remind -> RuleAction.Remind(text.trim())
        ActionKind.Checklist -> RuleAction.Checklist(items.lines().map { it.trim() }.filter { it.isNotEmpty() })
    }

    companion object {
        fun of(rule: Rule): RuleForm {
            val base = RuleForm(name = rule.name, enabled = rule.enabled)
            val withTrigger = when (val trigger = rule.trigger) {
                is Trigger.Place -> base.copy(
                    trigger = TriggerKind.Place,
                    latitude = trigger.latitude.toString(),
                    longitude = trigger.longitude.toString(),
                    radius = trigger.radiusM.toString(),
                    placeTransition = trigger.transition,
                )
                is Trigger.Time -> base.copy(
                    trigger = TriggerKind.Time,
                    hour = (trigger.minuteOfDay / MINUTES_PER_HOUR).toString(),
                    minute = (trigger.minuteOfDay % MINUTES_PER_HOUR).toString(),
                    days = trigger.days,
                )
                is Trigger.Wifi -> base.copy(trigger = TriggerKind.Wifi, wifiTransition = trigger.transition)
            }
            return when (val action = rule.action) {
                RuleAction.ShowTasks -> withTrigger.copy(action = ActionKind.ShowTasks)
                is RuleAction.Remind -> withTrigger.copy(action = ActionKind.Remind, text = action.text)
                is RuleAction.Checklist -> withTrigger.copy(action = ActionKind.Checklist, items = action.items.joinToString("\n"))
            }
        }

        /** A place rule that asks for the keys, wallet and phone on the way out; the user adds the place. */
        fun leavingHome() = RuleForm(
            name = "Leaving home",
            trigger = TriggerKind.Place,
            placeTransition = PlaceTransition.Leave,
            action = ActionKind.Checklist,
            items = LEAVING_HOME_ITEMS,
        )
    }
}

sealed interface FormResult {
    data class Valid(val rule: Rule) : FormResult

    data class Invalid(val message: String) : FormResult
}
