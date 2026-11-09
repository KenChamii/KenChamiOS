package com.kenchami.kenchamios.context

/** A place rule as the geofence registration needs it. */
data class PlaceFence(val ruleId: String, val place: Trigger.Place)

/** The Android pieces the rules use; the coordinator and the runner only know these. */
interface Geofences {
    /** Makes [places] the only registered geofences; returns a sentence for the user when registering failed. */
    suspend fun replaceAll(places: List<PlaceFence>): String?

    suspend fun clear()
}

interface TimePlan {
    /** Wakes the app once at about [epochMs]; null cancels. */
    fun wakeAt(epochMs: Long?)
}

interface WifiWatch {
    fun start()

    fun stop()

    fun connected(): Boolean
}

interface ContextPermissions {
    fun location(): Boolean

    /** True where the system has no separate background step (before Android 10). */
    fun backgroundLocation(): Boolean

    fun notifications(): Boolean
}

fun interface DueTasks {
    suspend fun titles(): List<String>
}

fun interface RuleNotifier {
    fun post(ruleId: String, title: String, text: String)
}
