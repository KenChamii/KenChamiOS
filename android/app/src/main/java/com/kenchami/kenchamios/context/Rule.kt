package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.model.WireEnum
import java.time.DayOfWeek

enum class PlaceTransition(override val wire: String) : WireEnum {
    Arrive("arrive"),
    Leave("leave"),
}

enum class WifiTransition(override val wire: String) : WireEnum {
    Connect("connect"),
    Disconnect("disconnect"),
}

sealed interface Trigger {
    data class Place(val latitude: Double, val longitude: Double, val radiusM: Int, val transition: PlaceTransition) : Trigger

    /** [minuteOfDay] is local time, 0 to 1439. */
    data class Time(val minuteOfDay: Int, val days: Set<DayOfWeek>) : Trigger

    data class Wifi(val transition: WifiTransition) : Trigger
}

sealed interface RuleAction {
    data object ShowTasks : RuleAction

    data class Remind(val text: String) : RuleAction

    data class Checklist(val items: List<String>) : RuleAction
}

data class Rule(val id: String, val name: String, val enabled: Boolean, val trigger: Trigger, val action: RuleAction)

/** What a receiver or worker reports; the engine decides which rules it concerns. */
sealed interface ContextEvent {
    data class PlaceChanged(val ruleId: String, val transition: PlaceTransition) : ContextEvent

    data class WifiChanged(val transition: WifiTransition) : ContextEvent

    data object TimeCheck : ContextEvent
}
