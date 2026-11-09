package com.kenchami.kenchamios.data.db

import androidx.sqlite.db.SupportSQLiteDatabase

/** The tables whose writes the sync sends; `shared/schema` marks them with x-synced and x-android. */
val SYNCED_TABLES = listOf(
    "activity",
    "checklist_items",
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

/**
 * Triggers keep `change_log` at one current entry per row, so no repository can forget to log a write.
 * The desktop has the same triggers (migrations 0011 and 0012).
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
     * The desktop's partial unique index on habit_checks (habit_id, day) WHERE deleted = 0.
     * Room cannot declare a partial index and refuses unknown ones, so triggers raise the same constraint error.
     */
    fun createUniqueLiveTriggers(db: SupportSQLiteDatabase) {
        for (event in listOf("INSERT", "UPDATE")) {
            db.execSQL(
                "CREATE TRIGGER IF NOT EXISTS trg_habit_checks_live_${event.lowercase()} BEFORE $event ON habit_checks " +
                    "WHEN NEW.deleted = 0 AND EXISTS (SELECT 1 FROM habit_checks WHERE habit_id = NEW.habit_id " +
                    "AND day = NEW.day AND deleted = 0 AND id != NEW.id) " +
                    "BEGIN SELECT RAISE(ABORT, 'UNIQUE constraint failed: habit_checks.habit_id, habit_checks.day'); END",
            )
        }
    }

    /** Rows that exist before the triggers do go out with the first sync. */
    fun backfill(db: SupportSQLiteDatabase, tables: List<String>) {
        for (table in tables) {
            db.execSQL("INSERT INTO change_log (table_name, row_id, hlc) SELECT '$table', id, updated_at FROM $table")
        }
    }
}
