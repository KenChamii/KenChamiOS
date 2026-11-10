package com.kenchami.kenchamios.data.db

import androidx.sqlite.db.SupportSQLiteDatabase

/** The tables whose writes the sync sends; `shared/schema` marks them with x-synced and x-android. */
val SYNCED_TABLES = listOf(
    "activity",
    "checklist_items",
    "daily_notes",
    "habit_checks",
    "habits",
    "media_entries",
    "media_items",
    "media_progress_log",
    "project_docs",
    "projects",
    "prompts",
    "save_points",
    "tasks",
    "time_entries",
)

/** The desktop's partial unique indexes `(columns) WHERE deleted = 0` that the phone enforces, by table. */
val UNIQUE_LIVE_KEYS = mapOf(
    "habit_checks" to listOf("habit_id", "day"),
    "daily_notes" to listOf("day"),
)

/**
 * Triggers keep `change_log` at one current entry per row, so no repository can forget to log a write.
 * The desktop has the same triggers (migrations 0011 to 0013).
 */
object ChangeLogSql {
    fun createTriggers(db: SupportSQLiteDatabase, tables: List<String> = SYNCED_TABLES) {
        for (table in tables) {
            for (event in listOf("INSERT", "UPDATE")) {
                db.execSQL(
                    "CREATE TRIGGER IF NOT EXISTS trg_${table}_log_${event.lowercase()} AFTER $event ON $table BEGIN " +
                        "DELETE FROM change_log WHERE table_name = '$table' AND row_id = NEW.id; " +
                        "INSERT INTO change_log (table_name, row_id, hlc) VALUES ('$table', NEW.id, NEW.updated_at); END",
                )
            }
        }
    }

    /**
     * The desktop's partial unique indexes, see [UNIQUE_LIVE_KEYS].
     * Room cannot declare a partial index and refuses unknown ones, so triggers raise the same constraint error.
     */
    fun createUniqueLiveTriggers(db: SupportSQLiteDatabase, tables: Collection<String> = UNIQUE_LIVE_KEYS.keys) {
        for (table in tables) {
            val columns = UNIQUE_LIVE_KEYS.getValue(table)
            val sameKey = columns.joinToString(" AND ") { "$it = NEW.$it" }
            val message = "UNIQUE constraint failed: " + columns.joinToString { "$table.$it" }
            for (event in listOf("INSERT", "UPDATE")) {
                db.execSQL(
                    "CREATE TRIGGER IF NOT EXISTS trg_${table}_live_${event.lowercase()} BEFORE $event ON $table " +
                        "WHEN NEW.deleted = 0 AND EXISTS (SELECT 1 FROM $table WHERE $sameKey AND deleted = 0 AND id != NEW.id) " +
                        "BEGIN SELECT RAISE(ABORT, '$message'); END",
                )
            }
        }
    }

    /** Rows that exist before the triggers do go out with the first sync. */
    fun backfill(db: SupportSQLiteDatabase, tables: List<String>) {
        for (table in tables) {
            db.execSQL("INSERT INTO change_log (table_name, row_id, hlc) SELECT '$table', id, updated_at FROM $table")
        }
    }
}
