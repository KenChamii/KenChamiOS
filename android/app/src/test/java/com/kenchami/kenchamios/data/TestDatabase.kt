package com.kenchami.kenchamios.data

import androidx.room.Room
import androidx.test.core.app.ApplicationProvider
import com.kenchami.kenchamios.TestClock
import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.core.Uuid7Generator
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.ChangeLogCallback

/** An in-memory database with the helpers a repository needs, on a clock the test moves. */
class TestDatabase(val clock: TestClock = TestClock()) {
    val hlc = Hlc(clock)
    val database: AppDatabase = Room.inMemoryDatabaseBuilder(
        ApplicationProvider.getApplicationContext(),
        AppDatabase::class.java,
    ).addCallback(ChangeLogCallback(hlc)).allowMainThreadQueries().build()
    val stamper = Stamper(clock, hlc, Uuid7Generator(clock), database.settingDao())
    val activity = ActivityRecorder(stamper, database.activityDao())

    /** The number a query like `SELECT COUNT(*) ...` returns. */
    fun count(sql: String): Int = database.openHelper.writableDatabase.query(sql).use { cursor ->
        cursor.moveToFirst()
        cursor.getInt(0)
    }

    fun close() = database.close()
}
