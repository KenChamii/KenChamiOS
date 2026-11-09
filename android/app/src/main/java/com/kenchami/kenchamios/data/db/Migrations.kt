package com.kenchami.kenchamios.data.db

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

private val HABIT_TABLES = listOf("habit_checks", "habits")

// a migration only touches the tables that exist at its version
private val TABLES_OF_VERSION_2 = SYNCED_TABLES - HABIT_TABLES

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
        ChangeLogSql.createTriggers(db, TABLES_OF_VERSION_2)
        ChangeLogSql.backfill(db, TABLES_OF_VERSION_2)
    }
}

/** Repeating tasks and habits, the same schema as desktop migration 0012. */
val MIGRATION_2_3 = object : Migration(2, 3) {
    override fun migrate(db: SupportSQLiteDatabase) {
        db.execSQL("ALTER TABLE `tasks` ADD COLUMN `recurrence` TEXT")
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `habits` (`id` TEXT NOT NULL, `created_at` INTEGER NOT NULL, " +
                "`updated_at` INTEGER NOT NULL, `device_id` TEXT NOT NULL, `deleted` INTEGER NOT NULL, " +
                "`name` TEXT NOT NULL, PRIMARY KEY(`id`))",
        )
        db.execSQL(
            "CREATE TABLE IF NOT EXISTS `habit_checks` (`id` TEXT NOT NULL, `created_at` INTEGER NOT NULL, " +
                "`updated_at` INTEGER NOT NULL, `device_id` TEXT NOT NULL, `deleted` INTEGER NOT NULL, " +
                "`habit_id` TEXT NOT NULL, `day` TEXT NOT NULL, PRIMARY KEY(`id`), " +
                "FOREIGN KEY(`habit_id`) REFERENCES `habits`(`id`) ON UPDATE NO ACTION ON DELETE NO ACTION )",
        )
        db.execSQL("CREATE INDEX IF NOT EXISTS `index_habit_checks_habit_id_day` ON `habit_checks` (`habit_id`, `day`)")
        ChangeLogSql.createTriggers(db, HABIT_TABLES)
        ChangeLogSql.createUniqueLiveTriggers(db)
    }
}
