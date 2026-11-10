package com.kenchami.kenchamios.sync

import com.kenchami.kenchamios.TestClock
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SyncValidatorTest {
    private val clock = TestClock()
    private val validator = SyncValidator(SyncFixtures.schema, clock)

    private fun rejects(changes: List<Change>) = try {
        validator.validate(changes)
        false
    } catch (_: RejectedException) {
        true
    }

    @Test
    fun theSharedFixturesAreValid() {
        validator.validate(SyncFixtures.changes("push-valid"))
        validator.validate(SyncFixtures.changes("pull-valid"))
    }

    @Test
    fun everySharedRejectedCaseIsRefused() {
        for (case in SyncFixtures.objects(SyncFixtures.array("rejected"))) {
            val changes = SyncFixtures.changes(case.getJSONArray("changes"))
            assertTrue("${case.getString("name")} was accepted", rejects(changes))
        }
    }

    @Test
    fun theClockSkewHasAHardEdge() {
        val change = SyncFixtures.changes("push-valid").first()
        val edge = (clock.millis() + 60 * 60 * 1000L) shl 16

        change.row.put("updated_at", edge)
        validator.validate(listOf(change))

        change.row.put("updated_at", edge + (1L shl 16))
        assertTrue(rejects(listOf(change)))
    }

    @Test
    fun aBatchOverTheLimitIsRefused() {
        val one = SyncFixtures.changes("push-valid").first()
        assertTrue(rejects(List(MAX_CHANGES + 1) { one }))
    }

    @Test
    fun decimalNumbersAreNotIntegers() {
        val change = SyncFixtures.changes("push-valid").first()
        change.row.put("created_at", 1.5)
        assertTrue(rejects(listOf(change)))
    }

    @Test
    fun idsAreLimitedToSafeCharacters() {
        assertTrue(isValidId("0190f0a2-7a1c-7c3e-9b52-1d3f6a8e4c10"))
        assertEquals(listOf(false, false, false), listOf("", "a b", "a".repeat(65)).map(::isValidId))
    }

    @Test
    fun theSchemaListsTheFourteenSyncedTables() {
        assertEquals(
            listOf(
                "activity", "checklist_items", "daily_notes", "habit_checks", "habits", "media_entries", "media_items",
                "media_progress_log", "project_docs", "projects", "prompts", "save_points", "tasks", "time_entries",
            ),
            SyncFixtures.schema.all.map { it.name }.sorted(),
        )
        assertNull(SyncFixtures.schema.table("settings"))
    }
}
