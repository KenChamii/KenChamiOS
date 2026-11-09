package com.kenchami.kenchamios.data.db

import androidx.sqlite.db.SupportSQLiteDatabase

/** The tables whose writes the sync sends; `shared/schema` marks them with x-synced and x-android. */
val SYNCED_TABLES = listOf(
    "activity",
    "checklist_items",
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

/**
 * Triggers keep `change_log` at one current entry per row, so no repository can forget to log a write.
 * The desktop has the same triggers (migration 0011).
 */
object ChangeLogSql {
    fun createTriggers(db: SupportSQLiteDatabase) {
        for (table in SYNCED_TABLES) {
            for (event in listOf("INSERT", "UPDATE")) {
                db.execSQL(
                    "CREATE TRIGGER IF NOT EXISTS trg_${table}_log_${event.lowercase()} AFTER $event ON $table BEGIN " +
                        "DELETE FROM change_log WHERE table_name = '$table' AND row_id = NEW.id; " +
                        "INSERT INTO change_log (table_name, row_id, hlc) VALUES ('$table', NEW.id, NEW.updated_at); END",
                )
            }
        }
    }

    /** Rows that exist before the triggers do go out with the first sync. */
    fun backfill(db: SupportSQLiteDatabase) {
        for (table in SYNCED_TABLES) {
            db.execSQL("INSERT INTO change_log (table_name, row_id, hlc) SELECT '$table', id, updated_at FROM $table")
        }
    }
}
