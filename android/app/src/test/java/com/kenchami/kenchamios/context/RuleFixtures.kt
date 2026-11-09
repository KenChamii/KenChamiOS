package com.kenchami.kenchamios.context

import java.time.DayOfWeek

// obviously fake values: the equator and the prime meridian
fun placeRule(
    id: String = "p1",
    transition: PlaceTransition = PlaceTransition.Leave,
    enabled: Boolean = true,
    action: RuleAction = RuleAction.Remind("Keys"),
) = Rule(id, "Place $id", enabled, Trigger.Place(0.0, 0.0, 150, transition), action)

fun timeRule(id: String = "t1", minuteOfDay: Int = 8 * 60, days: Set<DayOfWeek> = DayOfWeek.entries.toSet(), enabled: Boolean = true) =
    Rule(id, "Time $id", enabled, Trigger.Time(minuteOfDay, days), RuleAction.Remind("Stand up"))

fun wifiRule(id: String = "w1", transition: WifiTransition = WifiTransition.Connect, enabled: Boolean = true) =
    Rule(id, "Wifi $id", enabled, Trigger.Wifi(transition), RuleAction.ShowTasks)
