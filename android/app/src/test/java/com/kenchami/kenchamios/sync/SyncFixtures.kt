package com.kenchami.kenchamios.sync

import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/** The shared files the desktop tests read as well. */
object SyncFixtures {
    private val schemaDir = File("../../shared/schema")
    private val fixtureDir = File("../../shared/fixtures/sync")

    val schema: SyncSchema by lazy {
        SyncSchema.parse(File(schemaDir, "tables.schema.json").readText(), File(schemaDir, "enums.schema.json").readText())
    }

    fun text(name: String): String = File(fixtureDir, "$name.json").readText()

    fun obj(name: String) = JSONObject(text(name))

    fun array(name: String) = JSONArray(text(name))

    fun changes(array: JSONArray): List<Change> = List(array.length()) {
        val item = array.getJSONObject(it)
        Change(item.getString("table"), item.getJSONObject("row"))
    }

    fun changes(name: String): List<Change> = changes(obj(name).getJSONArray("changes"))

    fun objects(array: JSONArray): List<JSONObject> = List(array.length()) { array.getJSONObject(it) }
}
