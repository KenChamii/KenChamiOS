package com.kenchami.kenchamios.sync

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.time.Clock

// a remote hlc this far ahead of the local clock is refused, see ADR 0005
private const val MAX_SKEW_MS = 60 * 60 * 1000L
private const val HLC_COUNTER_BITS = 16
private val ID_PATTERN = Regex("^[A-Za-z0-9-]{1,64}$")

fun isValidId(value: String) = ID_PATTERN.matches(value)

/** Treats everything from the peer as hostile: a batch passes in full or not at all. */
class SyncValidator(private val schema: SyncSchema, private val clock: Clock) {
    fun validate(changes: List<Change>) {
        if (changes.size > MAX_CHANGES) throw RejectedException("too many changes in one batch")
        for (change in changes) {
            val table = schema.table(change.table) ?: throw RejectedException("table is not synced")
            validateRow(table, change.row)
        }
    }

    private fun validateRow(table: SyncTable, row: JSONObject) {
        if (row.length() != table.columns.size) throw RejectedException("${table.name}: wrong set of columns")
        for (column in table.columns) {
            if (!row.has(column.name)) throw RejectedException("${table.name}: missing column ${column.name}")
            validateValue(table, column, row.get(column.name))
        }
        for (key in listOf("id", "device_id")) {
            if (!isValidId(row.getString(key))) throw RejectedException("${table.name}: invalid $key")
        }
        val hlc = row.getLong("updated_at")
        if (hlc < 0) throw RejectedException("${table.name}: invalid updated_at")
        if ((hlc shr HLC_COUNTER_BITS) > clock.millis() + MAX_SKEW_MS) {
            throw RejectedException("${table.name}: updated_at is too far ahead")
        }
    }

    private fun validateValue(table: SyncTable, column: SyncColumn, value: Any) {
        fun bad(what: String): Nothing = throw RejectedException("${table.name}.${column.name}: $what")
        if (value == JSONObject.NULL) {
            if (!column.nullable) bad("null")
            return
        }
        when (column.kind) {
            ColumnKind.INTEGER -> {
                val number = (value as? Int)?.toLong() ?: value as? Long ?: bad("not an integer")
                if (column.allowedInt?.contains(number) == false) bad("value not allowed")
            }
            ColumnKind.TEXT -> {
                val text = value as? String ?: bad("not a string")
                if (column.allowedText?.contains(text) == false) bad("value not allowed")
                if (column.stringArray && !isStringArray(text)) bad("not a json array of strings")
            }
        }
    }

    private fun isStringArray(text: String): Boolean = try {
        val array = JSONArray(text)
        (0 until array.length()).all { array.get(it) is String }
    } catch (_: JSONException) {
        false
    }
}
