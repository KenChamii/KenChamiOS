package com.kenchami.kenchamios.context

import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

/** Makes the phone's geofences, wake-up and Wi-Fi watch match the enabled rules. Safe to call as often as needed. */
@Singleton
class ContextCoordinator @Inject constructor(
    private val rules: RuleRepository,
    private val geofences: Geofences,
    private val timePlan: TimePlan,
    private val wifi: WifiWatch,
    private val permissions: ContextPermissions,
    private val clock: Clock,
) {
    /** Returns a sentence when the places could not be registered, null otherwise. */
    suspend fun refresh(): String? {
        val active = if (rules.enabled()) rules.rules().filter { it.enabled } else emptyList()
        planTime(active)
        planWifi(active)
        return planPlaces(active)
    }

    private fun planTime(active: List<Rule>) {
        val now = clock.millis()
        val next = active.mapNotNull {
            (it.trigger as? Trigger.Time)?.let { t -> RuleSchedule.next(now, t.minuteOfDay, t.days, clock.zone) }
        }
        timePlan.wakeAt(next.minOrNull())
    }

    private suspend fun planWifi(active: List<Rule>) {
        if (active.none { it.trigger is Trigger.Wifi }) return wifi.stop()
        wifi.start()
        if (rules.wifiConnected() == null) rules.setWifiConnected(wifi.connected())
    }

    private suspend fun planPlaces(active: List<Rule>): String? {
        val places = active.mapNotNull { rule -> (rule.trigger as? Trigger.Place)?.let { PlaceFence(rule.id, it) } }.take(MAX_PLACE_RULES)
        if (places.isEmpty() || !permissions.location() || !permissions.backgroundLocation()) {
            geofences.clear()
            return null
        }
        return geofences.replaceAll(places)
    }
}
