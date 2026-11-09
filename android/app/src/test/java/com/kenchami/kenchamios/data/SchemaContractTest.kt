package com.kenchami.kenchamios.data

import android.database.Cursor
import com.kenchami.kenchamios.data.model.DocKind
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.SavePointSource
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TimeEntryKind
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.data.model.WireEnum
import org.json.JSONArray
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.io.File

/** Room and the Kotlin enums must describe the same data as `shared/schema`. */
@RunWith(RobolectricTestRunner::class)
class SchemaContractTest {
    private val schemaDir = File("../../shared/schema")
    private val tables = read("tables.schema.json").getJSONObject("\$defs")
    private val enums = read("enums.schema.json").getJSONObject("\$defs")
    private lateinit var env: TestDatabase

    private val kotlinEnums: Map<String, List<WireEnum>> = mapOf(
        "stage" to Stage.entries,
        "doc_kind" to DocKind.entries,
        "save_point_source" to SavePointSource.entries,
        "task_status" to TaskStatus.entries,
        "priority" to Priority.entries,
        "time_entry_kind" to TimeEntryKind.entries,
        "timer_phase" to TimerPhase.entries,
        "timer_status" to TimerStatus.entries,
        "media_type" to MediaType.entries,
        "media_status" to MediaStatus.entries,
        "media_source" to MediaSource.entries,
    )

    @Before
    fun open() {
        env = TestDatabase()
    }

    @After
    fun close() = env.close()

    @Test
    fun roomCreatesTheAndroidTablesOfTheSchema() {
        val expected = tables.keys().asSequence().filter { tables.getJSONObject(it).getBoolean("x-android") }.toSet()
        assertEquals(expected, roomTables())
    }

    @Test
    fun everyColumnMatchesTheSchema() {
        for (table in roomTables()) {
            val definition = tables.getJSONObject(table)
            val properties = definition.getJSONObject("properties")
            val required = strings(definition.getJSONArray("required")).toSet()
            val columns = columnsOf(table)
            assertEquals("$table columns", properties.keys().asSequence().toSet(), columns.keys)
            for ((name, column) in columns) {
                assertEquals("$table.$name not null", name in required, column.notNull)
                assertEquals("$table.$name type", sqlType(properties.getJSONObject(name)), column.type)
            }
        }
    }

    @Test
    fun kotlinEnumsMatchTheSchema() {
        val used = enumIdsOfAndroidTables()
        assertEquals("a schema enum has no Kotlin enum", used, used.intersect(kotlinEnums.keys))
        for (id in used) {
            val expected = strings(enums.getJSONObject(id).getJSONArray("enum"))
            assertEquals("enum $id", expected, kotlinEnums.getValue(id).map { it.wire })
        }
    }

    @Test
    fun theInboxHasTheWellKnownIdNameAndSlug() {
        val inbox = read("well-known.json").getJSONObject("inbox_project")
        assertEquals(
            listOf(inbox.getString("id"), inbox.getString("name"), inbox.getString("slug")),
            listOf(Inbox.ID, Inbox.NAME, Inbox.SLUG),
        )
    }

    private fun read(file: String) = JSONObject(File(schemaDir, file).readText())

    private fun strings(array: JSONArray) = List(array.length()) { array.getString(it) }

    private fun roomTables(): Set<String> = query(
        "SELECT name FROM sqlite_master WHERE type = 'table' " +
            "AND name NOT IN ('android_metadata', 'room_master_table', 'sqlite_sequence')",
    ) { it.getString(0) }.toSet()

    private class Column(val type: String, val notNull: Boolean)

    private fun columnsOf(table: String): Map<String, Column> = query("SELECT name, type, \"notnull\" FROM pragma_table_info('$table')") {
        it.getString(0) to Column(it.getString(1), it.getInt(2) == 1)
    }.toMap()

    private fun <T> query(sql: String, row: (Cursor) -> T): List<T> = env.database.openHelper.writableDatabase.query(sql).use { cursor ->
        generateSequence { if (cursor.moveToNext()) row(cursor) else null }.toList()
    }

    // a nullable column has a type list or an anyOf with null, an enum has no type of its own and is text
    private fun sqlType(property: JSONObject): String {
        val type = property.optJSONArray("type")?.getString(0) ?: property.optString("type")
        return if (type == "integer") "INTEGER" else "TEXT"
    }

    private fun enumIdsOfAndroidTables(): Set<String> {
        val reference = Regex("enums.schema.json#/\\\$defs/(\\w+)")
        return tables.keys().asSequence()
            .map { tables.getJSONObject(it) }
            .filter { it.getBoolean("x-android") }
            .flatMap { reference.findAll(it.getJSONObject("properties").toString()).map { match -> match.groupValues[1] } }
            .toSet()
    }
}
