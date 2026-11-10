package com.kenchami.kenchamios.sync

import android.database.Cursor
import android.database.sqlite.SQLiteConstraintException
import androidx.sqlite.db.SupportSQLiteDatabase
import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.time.Clock

private const val TOMBSTONE_AGE_MS = 30L * 24 * 60 * 60 * 1000
private const val HLC_COUNTER_BITS = 16

// children before parents, so most rows can go in one run
private val PURGE_ORDER = listOf(
    "activity",
    "daily_notes",
    "habit_checks",
    "habits",
    "media_progress_log",
    "media_entries",
    "media_items",
    "checklist_items",
    "time_entries",
    "save_points",
    "project_docs",
    "prompts",
    "tasks",
    "projects",
)

/** Reads the changes to send and applies the ones received, on the same database as the repositories. */
class SyncStore(
    private val database: AppDatabase,
    private val schema: SyncSchema,
    private val hlc: Hlc,
    private val clock: Clock,
    private val stamper: Stamper,
) {
    private val validator = SyncValidator(schema, clock)

    /** The current rows changed after [since]; rows last written by [skipDevice] are left out. */
    suspend fun readPage(since: Long, limit: Int, skipDevice: String): ChangePage = withContext(Dispatchers.IO) {
        val db = database.openHelper.writableDatabase
        val entries = db.query(
            "SELECT seq, table_name, row_id FROM change_log WHERE seq > ? ORDER BY seq LIMIT ?",
            arrayOf<Any?>(
                since,
                limit + 1,
            ),
        )
            .use { cursor -> rows(cursor) { Triple(it.getLong(0), it.getString(1), it.getString(2)) } }
        val taken = entries.take(limit)
        val changes = taken.mapNotNull { (_, tableName, rowId) -> readChange(db, tableName, rowId, skipDevice) }
        ChangePage(changes, taken.lastOrNull()?.first ?: since, entries.size > limit)
    }

    private fun <T> rows(cursor: Cursor, read: (Cursor) -> T): List<T> =
        generateSequence { if (cursor.moveToNext()) read(cursor) else null }.toList()

    private fun readChange(db: SupportSQLiteDatabase, tableName: String, rowId: String, skipDevice: String): Change? {
        val table = schema.table(tableName) ?: return null
        val row = db.query(table.selectSql, arrayOf(rowId)).use { cursor ->
            if (cursor.moveToFirst()) toJson(table, cursor) else null
        } ?: return null
        return if (row.getString("device_id") == skipDevice) null else Change(table.name, row)
    }

    private fun toJson(table: SyncTable, cursor: Cursor): JSONObject {
        val row = JSONObject()
        table.columns.forEachIndexed { index, column ->
            val value: Any = when {
                table.isLocal(column.name) || cursor.isNull(index) -> JSONObject.NULL
                column.kind == ColumnKind.INTEGER -> cursor.getLong(index)
                else -> cursor.getString(index)
            }
            row.put(column.name, value)
        }
        return row
    }

    /** Validates the batch and stores it in one transaction. Newer versions win, see ADR 0005. */
    suspend fun apply(changes: List<Change>): ApplyOutcome {
        validator.validate(changes)
        if (changes.isEmpty()) return ApplyOutcome(0, 0)
        val deviceId = stamper.deviceId()
        val outcome = withContext(Dispatchers.IO) {
            val db = database.openHelper.writableDatabase
            // a page can carry a child before its parent, the next page completes the pair
            db.execSQL("PRAGMA foreign_keys = OFF")
            try {
                db.beginTransaction()
                try {
                    applyAll(db, changes, deviceId).also { db.setTransactionSuccessful() }
                } finally {
                    db.endTransaction()
                }
            } finally {
                db.execSQL("PRAGMA foreign_keys = ON")
            }
        }
        hlc.merge(changes.maxOf { it.updatedAt })
        return outcome
    }

    private fun applyAll(db: SupportSQLiteDatabase, changes: List<Change>, deviceId: String): ApplyOutcome {
        var applied = 0
        for (change in changes) {
            val table = checkNotNull(schema.table(change.table))
            resolveUniqueKey(db, table, change.row, deviceId)
            if (store(db, table, change.row)) applied++
        }
        return ApplyOutcome(applied, changes.size - applied)
    }

    private fun store(db: SupportSQLiteDatabase, table: SyncTable, row: JSONObject): Boolean {
        val stored = db.query("SELECT updated_at, device_id FROM ${table.name} WHERE id = ?", arrayOf(row.getString("id"))).use { cursor ->
            if (cursor.moveToFirst()) cursor.getLong(0) to cursor.getString(1) else null
        }
        if (stored == null) {
            db.execSQL(table.insertSql, table.sharedColumns.map { value(row, it) }.toTypedArray())
            return true
        }
        if (!isNewer(row.getLong("updated_at") to row.getString("device_id"), stored)) return false
        val values = table.sharedColumns.filter { it.name != "id" }.map { value(row, it) } + row.getString("id")
        db.execSQL(table.updateSql, values.toTypedArray())
        return true
    }

    private fun value(row: JSONObject, column: SyncColumn): Any? = when {
        row.isNull(column.name) -> null
        column.kind == ColumnKind.INTEGER -> row.getLong(column.name)
        else -> row.getString(column.name)
    }

    private fun isNewer(a: Pair<Long, String>, b: Pair<Long, String>) = a.first > b.first || (a.first == b.first && a.second > b.second)

    // two live rows with the same key cannot exist: the newer version keeps the key, the other is deleted
    private fun resolveUniqueKey(db: SupportSQLiteDatabase, table: SyncTable, row: JSONObject, deviceId: String) {
        if (table.uniqueLive.isEmpty() || row.getLong("deleted") != 0L || table.uniqueLive.any { row.isNull(it) }) return
        val keyColumns = table.sharedColumns.filter { it.name in table.uniqueLive }
        val sql = "SELECT id, updated_at, device_id FROM ${table.name} WHERE deleted = 0 AND id != ? AND " +
            keyColumns.joinToString(" AND ") { "${it.name} = ?" }
        val arguments = (listOf<Any?>(row.getString("id")) + keyColumns.map { value(row, it) }).toTypedArray()
        val rivals = db.query(sql, arguments).use { cursor ->
            rows(cursor) { Triple(it.getString(0), it.getLong(1), it.getString(2)) }
        }
        val incoming = row.getLong("updated_at") to row.getString("device_id")
        for ((id, updatedAt, device) in rivals) {
            if (isNewer(incoming, updatedAt to device)) {
                db.execSQL(
                    "UPDATE ${table.name} SET deleted = 1, updated_at = ?, device_id = ? WHERE id = ?",
                    arrayOf<Any?>(hlc.now(), deviceId, id),
                )
            } else {
                row.put("deleted", 1)
            }
        }
    }

    /** Removes tombstones older than 30 days that were already sent; returns how many went. */
    suspend fun purgeSentTombstones(sentUpTo: Long): Int = withContext(Dispatchers.IO) {
        val db = database.openHelper.writableDatabase
        val cutoff = (clock.millis() - TOMBSTONE_AGE_MS) shl HLC_COUNTER_BITS
        var removed = 0
        for (table in PURGE_ORDER) {
            val ids = db.query(
                "SELECT t.id FROM $table t JOIN change_log l ON l.table_name = ? AND l.row_id = t.id " +
                    "WHERE t.deleted = 1 AND t.updated_at < ? AND l.seq <= ?",
                arrayOf<Any?>(table, cutoff, sentUpTo),
            ).use { cursor -> rows(cursor) { it.getString(0) } }
            for (id in ids) {
                try {
                    db.execSQL("DELETE FROM $table WHERE id = ?", arrayOf(id))
                } catch (_: SQLiteConstraintException) {
                    // still referenced, it goes on the next run
                    continue
                }
                db.execSQL("DELETE FROM change_log WHERE table_name = ? AND row_id = ?", arrayOf(table, id))
                removed++
            }
        }
        removed
    }
}
