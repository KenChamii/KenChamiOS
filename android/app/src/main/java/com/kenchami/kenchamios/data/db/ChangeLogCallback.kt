package com.kenchami.kenchamios.data.db

import androidx.room.RoomDatabase
import androidx.sqlite.db.SupportSQLiteDatabase
import com.kenchami.kenchamios.core.Hlc

/** Installs the triggers Room cannot declare on a new database and moves the clock past every logged change when it opens. */
class ChangeLogCallback(private val hlc: Hlc) : RoomDatabase.Callback() {
    override fun onCreate(db: SupportSQLiteDatabase) {
        ChangeLogSql.createTriggers(db)
        ChangeLogSql.createUniqueLiveTriggers(db)
    }

    override fun onOpen(db: SupportSQLiteDatabase) {
        db.query("SELECT MAX(hlc) FROM change_log").use { cursor ->
            if (cursor.moveToFirst() && !cursor.isNull(0)) hlc.merge(cursor.getLong(0))
        }
    }
}
