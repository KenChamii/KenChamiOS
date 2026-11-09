package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.data.TestDatabase
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SyncStoreTest {
    private val env = TestDatabase()
    private val store = SyncStore(env.database, SyncFixtures.schema, env.hlc, env.clock, env.stamper)

    @After
    fun close() = env.close()

    private fun change(table: String, row: JSONObject) = Change(table, JSONObject(row.toString()))

    private fun TestDatabase.text(sql: String): String = database.openHelper.writableDatabase.query(sql).use { cursor ->
        cursor.moveToFirst()
        cursor.getString(0)
    }

    private fun query(sql: String) = env.text(sql)

    @Test
    fun sharedConflictCasesResolveAsSpecified() = runBlocking {
        for (case in SyncFixtures.objects(SyncFixtures.array("conflicts"))) {
            val local = TestDatabase()
            val localStore = SyncStore(local.database, SyncFixtures.schema, local.hlc, local.clock, local.stamper)
            val table = case.getString("table")
            localStore.apply(listOf(change(table, case.getJSONObject("stored"))))

            val outcome = localStore.apply(listOf(change(table, case.getJSONObject("incoming"))))

            val name = case.getString("name")
            assertEquals(name, case.getBoolean("applied"), outcome.applied == 1)
            assertEquals(name, case.getString("title"), local.text("SELECT title FROM tasks WHERE id = 't1'"))
            if (case.has("deleted")) {
                val deleted = local.count("SELECT deleted FROM tasks WHERE id = 't1'")
                assertEquals(name, case.getInt("deleted"), deleted)
            }
            local.close()
        }
    }

    @Test
    fun sharedUniqueKeyCasesKeepOneLiveRow() = runBlocking {
        for (case in SyncFixtures.objects(SyncFixtures.array("unique-keys"))) {
            val local = TestDatabase()
            val localStore = SyncStore(local.database, SyncFixtures.schema, local.hlc, local.clock, local.stamper)
            val table = case.getString("table")
            localStore.apply(listOf(change(table, case.getJSONObject("stored"))))
            localStore.apply(listOf(change(table, case.getJSONObject("incoming"))))

            val live = local.database.openHelper.writableDatabase.query(
                "SELECT id FROM $table WHERE deleted = 0 ORDER BY id",
            ).use { cursor ->
                generateSequence { if (cursor.moveToNext()) cursor.getString(0) else null }.toList()
            }

            val expected = case.getJSONArray("live").let { ids -> List(ids.length()) { ids.getString(it) } }
            assertEquals(case.getString("name"), expected, live)
            local.close()
        }
    }

    @Test
    fun aBatchWithOneBadChangeStoresNothing() = runBlocking {
        val changes = SyncFixtures.changes("push-valid")
        changes[3].row.put("updated_at", 9_000_000_000_000_000_000L)

        val result = runCatching { store.apply(changes) }

        assertTrue(result.exceptionOrNull() is RejectedException)
        assertEquals(0, env.count("SELECT COUNT(*) FROM projects") + env.count("SELECT COUNT(*) FROM tasks"))
    }

    @Test
    fun storedRowsComeBackExactlyAsSent() = runBlocking {
        val sent = SyncFixtures.changes("push-valid")
        store.apply(sent)

        val page = store.readPage(0, MAX_CHANGES, "nobody")

        assertEquals(sent.size, page.changes.size)
        for (original in sent) {
            val back = page.changes.single { it.table == original.table }
            for (key in original.row.keys()) {
                assertEquals(
                    "${original.table}.$key",
                    original.row.get(key).toString(),
                    back.row.get(key).toString(),
                )
            }
        }
    }

    @Test
    fun aDeviceDoesNotGetItsOwnRowsBack() = runBlocking {
        store.apply(SyncFixtures.changes("push-valid"))

        val page = store.readPage(0, MAX_CHANGES, "dev-phone")

        assertTrue(page.changes.isEmpty())
        assertTrue("the cursor still moves", page.next > 0)
    }

    @Test
    fun pagesAreCutAtTheLimit() = runBlocking {
        store.apply(SyncFixtures.changes("push-valid"))

        val first = store.readPage(0, 4, "nobody")
        val rest = store.readPage(first.next, MAX_CHANGES, "nobody")

        assertEquals(4, first.changes.size)
        assertTrue(first.hasMore)
        assertEquals(SyncFixtures.changes("push-valid").size - 4, rest.changes.size)
        assertEquals(false, rest.hasMore)
    }

    @Test
    fun theCoverCachePathOfThisDeviceSurvivesAnUpdate() = runBlocking {
        val item = SyncFixtures.changes("push-valid").single { it.table == "media_items" }
        store.apply(listOf(item))
        env.database.openHelper.writableDatabase.execSQL("UPDATE media_items SET cover_cache_path = 'mine.jpg' WHERE id = 'mi1'")
        val newer = change("media_items", item.row)
        newer.row.put("title", "Renamed").put("updated_at", item.updatedAt + 1_000_000)

        store.apply(listOf(newer))

        assertEquals("Renamed", query("SELECT title FROM media_items WHERE id = 'mi1'"))
        assertEquals("mine.jpg", query("SELECT cover_cache_path FROM media_items WHERE id = 'mi1'"))
    }

    @Test
    fun aChildMayArriveBeforeItsParentAndForeignKeysComeBack() = runBlocking {
        val all = SyncFixtures.changes("push-valid")
        val task = all.single { it.table == "tasks" }

        store.apply(listOf(task))
        store.apply(all.filter { it.table == "projects" })

        assertEquals(1, env.count("SELECT COUNT(*) FROM tasks"))
        assertEquals("1", query("PRAGMA foreign_keys"))
    }

    @Test
    fun theClockMovesPastWhatWasReceived() = runBlocking {
        val changes = SyncFixtures.changes("push-valid")
        store.apply(changes)

        assertTrue(env.stamper.stamp() > changes.maxOf { it.updatedAt })
    }

    @Test
    fun replayingABatchChangesNothing() = runBlocking {
        val changes = SyncFixtures.changes("push-valid")
        store.apply(changes)

        val again = store.apply(changes)

        assertEquals(0, again.applied)
        assertEquals(changes.size, again.ignored)
    }

    private fun age(table: String, id: String, days: Long) {
        val old = (env.clock.millis() - days * 24 * 60 * 60 * 1000) shl 16
        env.database.openHelper.writableDatabase.execSQL(
            "UPDATE $table SET deleted = 1, updated_at = ? WHERE id = ?",
            arrayOf<Any?>(old, id),
        )
    }

    @Test
    fun oldSentTombstonesArePurgedAndYoungOrUnsentOnesStay() = runBlocking {
        store.apply(SyncFixtures.changes("push-valid"))
        age("prompts", "pr1", 31)
        age("save_points", "s1", 29)
        age("media_progress_log", "ml1", 31)
        env.database.openHelper.writableDatabase.execSQL("UPDATE activity SET deleted = 1, updated_at = 1 WHERE id = 'a1'")
        val sentUpTo = query("SELECT MAX(seq) FROM change_log").toLong()

        val beforeSending = store.purgeSentTombstones(0)
        val removed = store.purgeSentTombstones(sentUpTo)

        assertEquals(0, beforeSending)
        assertEquals(3, removed)
        assertEquals(0, env.count("SELECT COUNT(*) FROM prompts"))
        assertEquals(0, env.count("SELECT COUNT(*) FROM media_progress_log"))
        assertEquals(1, env.count("SELECT COUNT(*) FROM save_points"))
        assertEquals(0, env.count("SELECT COUNT(*) FROM change_log WHERE row_id IN ('pr1', 'ml1', 'a1')"))
    }

    @Test
    fun aTombstoneThatIsStillReferencedIsKept() = runBlocking {
        store.apply(SyncFixtures.changes("push-valid"))
        age("projects", "p1", 31)
        val sentUpTo = query("SELECT MAX(seq) FROM change_log").toLong()

        store.purgeSentTombstones(sentUpTo)

        assertEquals(1, env.count("SELECT COUNT(*) FROM projects WHERE id = 'p1'"))
    }
}
