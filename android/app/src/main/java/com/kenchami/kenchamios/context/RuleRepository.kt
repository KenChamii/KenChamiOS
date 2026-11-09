package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.LocalSettings
import com.kenchami.kenchamios.data.db.dao.SettingValue
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

private const val RULE = "context.rule."
private const val FIRED = "context.fired."
private const val ENABLED = "context.enabled"
private const val WIFI = "context.wifi"

/** The rules, when they last fired and the switches, kept on this phone only. */
@Singleton
class RuleRepository @Inject constructor(private val settings: LocalSettings, private val clock: Clock) {
    fun observeRules(): Flow<List<Rule>> = settings.observePrefix(RULE).map(::decode)

    suspend fun rules(): List<Rule> = decode(settings.withPrefix(RULE))

    suspend fun get(id: String): Rule? = settings.get(RULE + id)?.let { RuleCodec.decodeRule(id, it) }

    /** A saved time rule starts counting from now, so editing it never fires a time that already passed. */
    suspend fun save(rule: Rule) {
        settings.put(RULE + rule.id, RuleCodec.encode(rule))
        if (rule.trigger is Trigger.Time) markFired(rule.id, clock.millis())
    }

    suspend fun delete(id: String) {
        settings.delete(RULE + id)
        settings.delete(FIRED + id)
        settings.delete(RUN + id)
    }

    suspend fun lastFired(): Map<String, Long> =
        settings.withPrefix(FIRED).mapNotNull { row -> row.valueJson.toLongOrNull()?.let { row.key.removePrefix(FIRED) to it } }.toMap()

    suspend fun markFired(id: String, at: Long) = settings.put(FIRED + id, at.toString())

    fun observeEnabled(): Flow<Boolean> = settings.observePrefix(ENABLED).map { rows -> rows.firstOrNull()?.valueJson != "false" }

    suspend fun enabled(): Boolean = settings.get(ENABLED) != "false"

    suspend fun setEnabled(enabled: Boolean) = settings.put(ENABLED, enabled.toString())

    suspend fun wifiConnected(): Boolean? = settings.get(WIFI)?.toBooleanStrictOrNull()

    suspend fun setWifiConnected(connected: Boolean) = settings.put(WIFI, connected.toString())

    private fun decode(rows: List<SettingValue>): List<Rule> =
        rows.sortedBy { it.key }.mapNotNull { RuleCodec.decodeRule(it.key.removePrefix(RULE), it.valueJson) }
}
