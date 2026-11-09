package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.db.SYNCED_TABLES
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

// the change log is filled by triggers, these tests fail when a synced table is left out
@RunWith(RobolectricTestRunner::class)
class ChangeLogTest {
    private val env = TestDatabase()
    private val store = SyncStore(env.database, SyncFixtures.schema, env.hlc, env.clock, env.stamper)

    @After
    fun close() = env.close()

    private fun rows(sql: String): List<List<String>> = env.database.openHelper.writableDatabase.query(sql).use { cursor ->
        generateSequence { if (cursor.moveToNext()) List(cursor.columnCount) { cursor.getString(it) } else null }.toList()
    }

    @Test
    fun theTriggerTablesAreTheSyncedAndroidTablesOfTheSchema() {
        assertEquals(SyncFixtures.schema.all.map { it.name }.sorted(), SYNCED_TABLES.sorted())
    }

    @Test
    fun everySyncedTableHasAnInsertAndAnUpdateTrigger() {
        val triggers = rows("SELECT name FROM sqlite_master WHERE type = 'trigger'").map { it[0] }.toSet()
        for (table in SYNCED_TABLES) {
            assertTrue("$table insert", "trg_${table}_log_insert" in triggers)
            assertTrue("$table update", "trg_${table}_log_update" in triggers)
        }
    }

    @Test
    fun everySyncedTableLogsAnInsertAndAnUpdate() = runBlocking {
        val changes = SyncFixtures.changes("push-valid")
        assertEquals(SYNCED_TABLES.size, changes.size)
        store.apply(changes)
        val db = env.database.openHelper.writableDatabase

        for (change in changes) {
            val logged = rows("SELECT seq, hlc FROM change_log WHERE table_name = '${change.table}' AND row_id = '${change.id}'")
            assertEquals("${change.table} insert", 1, logged.size)
            assertEquals(change.updatedAt, logged.single()[1].toLong())

            db.execSQL("UPDATE ${change.table} SET updated_at = updated_at + 5 WHERE id = ?", arrayOf<Any?>(change.id))
            val after = rows("SELECT seq, hlc FROM change_log WHERE table_name = '${change.table}' AND row_id = '${change.id}'").single()
            assertTrue("${change.table} update moves the entry", after[0].toLong() > logged.single()[0].toLong())
            assertEquals(change.updatedAt + 5, after[1].toLong())
        }
    }

    @Test
    fun repositoryWritesLeaveACurrentEntryForEveryRow() = runBlocking {
        val projects = ProjectRepository(env.database, env.stamper, env.activity)
        val tasks = TaskRepository(env.database, env.stamper, env.activity)
        val project = projects.create("Alpha")
        tasks.create(project, "Write")
        projects.delete(project)

        for (table in SYNCED_TABLES) {
            val unlogged = rows(
                "SELECT COUNT(*) FROM $table t WHERE NOT EXISTS " +
                    "(SELECT 1 FROM change_log l WHERE l.table_name = '$table' AND l.row_id = t.id AND l.hlc = t.updated_at)",
            )
            assertEquals("$table has rows without a current log entry", "0", unlogged.single()[0])
        }
    }

    @Test
    fun localOnlyTablesAreNeverLogged() = runBlocking {
        env.stamper.deviceId()

        assertEquals(emptyList<List<String>>(), rows("SELECT table_name FROM change_log"))
    }
}
