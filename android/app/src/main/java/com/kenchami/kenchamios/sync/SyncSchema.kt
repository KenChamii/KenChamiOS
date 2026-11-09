package com.kenchami.kenchamios.sync

import org.json.JSONArray
import org.json.JSONObject

enum class ColumnKind { INTEGER, TEXT }

class SyncColumn(
    val name: String,
    val kind: ColumnKind,
    val nullable: Boolean,
    val allowedText: Set<String>?,
    val allowedInt: Set<Long>?,
    val stringArray: Boolean,
)

class SyncTable(val name: String, val columns: List<SyncColumn>, private val localColumns: Set<String>, val uniqueLive: List<String>) {
    /** The columns that travel and are written on receive. */
    val sharedColumns: List<SyncColumn> = columns.filter { it.name !in localColumns }

    fun isLocal(column: String) = column in localColumns

    val selectSql = "SELECT ${columns.joinToString { it.name }} FROM $name WHERE id = ?"
    val insertSql = "INSERT INTO $name (${sharedColumns.joinToString { it.name }}) " +
        "VALUES (${sharedColumns.joinToString { "?" }})"
    val updateSql = "UPDATE $name SET ${sharedColumns.filter { it.name != "id" }.joinToString { "${it.name} = ?" }} WHERE id = ?"
}

/** The tables the phone syncs and their columns, read from the same `shared/schema` files as the desktop. */
class SyncSchema(private val tables: List<SyncTable>) {
    fun table(name: String): SyncTable? = tables.firstOrNull { it.name == name }

    val all: List<SyncTable> get() = tables

    companion object {
        fun parse(tablesJson: String, enumsJson: String): SyncSchema {
            val defs = JSONObject(tablesJson).getJSONObject("\$defs")
            val enums = JSONObject(enumsJson).getJSONObject("\$defs")
            val tables = defs.keys().asSequence()
                .map { it to defs.getJSONObject(it) }
                .filter { (_, def) -> def.optBoolean("x-synced") && def.optBoolean("x-android") }
                .map { (name, def) -> parseTable(name, def, enums) }
                .toList()
            return SyncSchema(tables)
        }

        private fun parseTable(name: String, def: JSONObject, enums: JSONObject): SyncTable {
            val properties = def.getJSONObject("properties")
            val columns = properties.keys().asSequence().map { parseColumn(it, properties.getJSONObject(it), enums) }.toList()
            return SyncTable(
                name,
                columns,
                strings(def.optJSONArray("x-local-columns")).toSet(),
                strings(def.optJSONArray("x-unique-live")),
            )
        }

        private fun parseColumn(name: String, property: JSONObject, enums: JSONObject): SyncColumn {
            val variants = property.optJSONArray("anyOf")?.let { any -> List(any.length()) { any.getJSONObject(it) } } ?: listOf(property)
            var kind = ColumnKind.TEXT
            var nullable = false
            var allowedText: Set<String>? = null
            var allowedInt: Set<Long>? = null
            for (variant in variants) {
                variant.optString("\$ref").takeIf { it.isNotEmpty() }?.let { reference ->
                    allowedText = strings(enums.getJSONObject(reference.substringAfterLast('/')).getJSONArray("enum")).toSet()
                }
                for (type in types(variant)) {
                    when (type) {
                        "null" -> nullable = true
                        "integer" -> kind = ColumnKind.INTEGER
                        "string" -> kind = ColumnKind.TEXT
                        else -> error("unsupported type $type in $name")
                    }
                }
                variant.optJSONArray("enum")?.let { values -> allowedInt = List(values.length()) { values.getLong(it) }.toSet() }
            }
            return SyncColumn(name, kind, nullable, allowedText, allowedInt, property.optString("x-encoding") == "json-string-array")
        }

        private fun types(variant: JSONObject): List<String> = when (val type = variant.opt("type")) {
            is String -> listOf(type)
            is JSONArray -> strings(type)
            else -> emptyList()
        }

        private fun strings(array: JSONArray?): List<String> = array?.let { List(it.length()) { i -> it.getString(i) } } ?: emptyList()
    }
}
