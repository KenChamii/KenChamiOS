package com.kenchami.kenchamios.data.db

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

/** Adds the change log for LAN sync, and lets a library item have more than one entry row (a deleted one and a live one). */
val MIGRATION_1_2 = object : Migration(1, 2) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `change_log` (`seq` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, " +
                "`table_name` TEXT NOT NULL, `row_id` TEXT NOT NULL, `hlc` INTEGER NOT NULL)",
        )
        db.execSQL("CREATE UNIQUE INDEX IF NOT EXISTS `index_change_log_table_name_row_id` ON `change_log` (`table_name`, `row_id`)")
        db.execSQL("DROP INDEX IF EXISTS `index_media_entries_media_item_id`")
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_media_entries_media_item_id` ON `media_entries` (`media_item_id`)")
        ChangeLogSql.createTriggers(db)
        ChangeLogSql.backfill(db)
    }
}
