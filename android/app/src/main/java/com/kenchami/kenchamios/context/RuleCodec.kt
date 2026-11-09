package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.model.fromWire
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.time.DateTimeException
import java.time.DayOfWeek

/** Rules and checklist runs as the JSON text kept in the local settings table. */
object RuleCodec {
    fun encode(rule: Rule): String = JSONObject()
        .put("name", rule.name)
        .put("enabled", rule.enabled)
        .put("trigger", trigger(rule.trigger))
        .put("action", action(rule.action))
        .toString()

    /** Null for text that is not a rule, so one damaged row cannot break the list. */
    fun decodeRule(id: String, json: String): Rule? = try {
        val root = JSONObject(json)
        Rule(
            id,
            root.getString("name"),
            root.getBoolean("enabled"),
            trigger(root.getJSONObject("trigger")),
            action(root.getJSONObject("action")),
        )
    } catch (_: JSONException) {
        null
    } catch (_: IllegalStateException) {
        null
    } catch (_: DateTimeException) {
        null
    }

    fun encode(run: ChecklistRun): String = JSONObject()
        .put("name", run.ruleName)
        .put("items", JSONArray(run.items.map { JSONObject().put("text", it.text).put("checked", it.checked) }))
        .toString()

    fun decodeRun(ruleId: String, json: String): ChecklistRun? = try {
        val root = JSONObject(json)
        val items = root.getJSONArray("items")
        ChecklistRun(
            ruleId,
            root.getString("name"),
            List(items.length()) {
                items.getJSONObject(it).let { item -> ChecklistItem(item.getString("text"), item.getBoolean("checked")) }
            },
        )
    } catch (_: JSONException) {
        null
    }

    private fun trigger(trigger: Trigger): JSONObject = when (trigger) {
        is Trigger.Place -> JSONObject().put("kind", "place").put("lat", trigger.latitude).put("lon", trigger.longitude)
            .put("radius_m", trigger.radiusM).put("when", trigger.transition.wire)
        is Trigger.Time -> JSONObject().put("kind", "time").put("minute", trigger.minuteOfDay)
            .put("days", JSONArray(trigger.days.map { it.value }.sorted()))
        is Trigger.Wifi -> JSONObject().put("kind", "wifi").put("when", trigger.transition.wire)
    }

    private fun trigger(json: JSONObject): Trigger = when (val kind = json.getString("kind")) {
        "place" -> Trigger.Place(
            json.getDouble("lat"),
            json.getDouble("lon"),
            json.getInt("radius_m"),
            fromWire<PlaceTransition>(json.getString("when")),
        )
        "time" -> {
            val days = json.getJSONArray("days")
            Trigger.Time(json.getInt("minute"), List(days.length()) { DayOfWeek.of(days.getInt(it)) }.toSet())
        }
        "wifi" -> Trigger.Wifi(fromWire<WifiTransition>(json.getString("when")))
        else -> error("unknown trigger $kind")
    }

    private fun action(action: RuleAction): JSONObject = when (action) {
        RuleAction.ShowTasks -> JSONObject().put("kind", "show_tasks")
        is RuleAction.Remind -> JSONObject().put("kind", "remind").put("text", action.text)
        is RuleAction.Checklist -> JSONObject().put("kind", "checklist").put("items", JSONArray(action.items))
    }

    private fun action(json: JSONObject): RuleAction = when (val kind = json.getString("kind")) {
        "show_tasks" -> RuleAction.ShowTasks
        "remind" -> RuleAction.Remind(json.getString("text"))
        "checklist" -> json.getJSONArray("items").let { items -> RuleAction.Checklist(List(items.length()) { items.getString(it) }) }
        else -> error("unknown action $kind")
    }
}
