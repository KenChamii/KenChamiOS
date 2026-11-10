package com.kenchami.kenchamios.data

import android.content.Context
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.driver.AndroidSQLiteDriver
import androidx.sqlite.execSQL
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.AppDatabase_Impl
import com.kenchami.kenchamios.data.db.MIGRATION_1_2
import com.kenchami.kenchamios.data.db.MIGRATION_2_3
import com.kenchami.kenchamios.data.db.MIGRATION_3_4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class MigrationTest {
    @get:Rule
    val helper = MigrationTestHelper(
        InstrumentationRegistry.getInstrumentation(),
        ApplicationProvider.getApplicationContext<Context>().getDatabasePath("migration-test"),
        AndroidSQLiteDriver(),
        AppDatabase::class,
        { AppDatabase_Impl() },
    )

    // the exported schema file must describe what Room expects, so a changed entity without a new version fails here
    @Test
    fun exportedVersionOneIsWhatRoomExpectsAndKeepsItsRows() {
        helper.createDatabase(1).apply {
            execSQL(
                "INSERT INTO projects (id, created_at, updated_at, device_id, deleted, name, slug, stage, " +
                    "description, tags, archived) VALUES ('p1', 1, 1, 'd', 0, 'Old', 'old', 'idea', '', '[]', 0)",
            )
            close()
        }

        val upgraded = helper.runMigrationsAndValidate(1, emptyList())
        val name = upgraded.prepare("SELECT name FROM projects WHERE id = 'p1'").use { statement ->
            statement.step()
            statement.getText(0)
        }
        upgraded.close()

        assertEquals("Old", name)
    }

    @Test
    fun versionTwoAddsTheChangeLogAndLogsTheRowsThatAlreadyExist() {
        helper.createDatabase(1).apply {
            execSQL(
                "INSERT INTO projects (id, created_at, updated_at, device_id, deleted, name, slug, stage, " +
                    "description, tags, archived) VALUES ('p1', 1, 77, 'd', 0, 'Old', 'old', 'idea', '', '[]', 0)",
            )
            close()
        }

        val upgraded = helper.runMigrationsAndValidate(2, listOf(MIGRATION_1_2))
        val logged = upgraded.prepare("SELECT table_name, row_id, hlc FROM change_log").use { statement ->
            statement.step()
            Triple(statement.getText(0), statement.getText(1), statement.getLong(2))
        }
        upgraded.execSQL("UPDATE projects SET name = 'New', updated_at = 99 WHERE id = 'p1'")
        val hlcAfterEdit = upgraded.prepare("SELECT hlc FROM change_log WHERE row_id = 'p1'").use { statement ->
            statement.step()
            statement.getLong(0)
        }
        upgraded.close()

        assertEquals(Triple("projects", "p1", 77L), logged)
        assertEquals(99L, hlcAfterEdit)
    }

    @Test
    fun versionThreeAddsRepeatsAndLoggedHabitsWithOneLiveCheckADay() {
        helper.createDatabase(2).apply {
            execSQL(
                "INSERT INTO projects (id, created_at, updated_at, device_id, deleted, name, slug, stage, " +
                    "description, tags, archived) VALUES ('p1', 1, 1, 'd', 0, 'Old', 'old', 'idea', '', '[]', 0)",
            )
            execSQL(
                "INSERT INTO tasks (id, created_at, updated_at, device_id, deleted, project_id, title, body_md, acceptance_md, " +
                    "related_files, status, priority, order_key, tags) VALUES ('t1', 1, 1, 'd', 0, 'p1', 'Old', '', '', '[]', " +
                    "'todo', 'medium', 'a0', '[]')",
            )
            close()
        }

        val upgraded = helper.runMigrationsAndValidate(3, listOf(MIGRATION_2_3))
        val recurrence = upgraded.prepare("SELECT recurrence FROM tasks WHERE id = 't1'").use { statement ->
            statement.step()
            statement.isNull(0)
        }
        upgraded.execSQL("INSERT INTO habits (id, created_at, updated_at, device_id, deleted, name) VALUES ('h1', 1, 60, 'd', 0, 'Read')")
        val check = "INSERT INTO habit_checks (id, created_at, updated_at, device_id, deleted, habit_id, day) VALUES "
        upgraded.execSQL("$check ('c1', 1, 61, 'd', 0, 'h1', '2026-11-10')")
        val duplicate = runCatching { upgraded.execSQL("$check ('c2', 1, 62, 'd', 0, 'h1', '2026-11-10')") }
        upgraded.execSQL("$check ('c3', 1, 63, 'd', 1, 'h1', '2026-11-10')")
        val logged = upgraded.prepare("SELECT table_name || ':' || hlc FROM change_log ORDER BY hlc").use { statement ->
            generateSequence { if (statement.step()) statement.getText(0) else null }.toList()
        }
        upgraded.close()

        assertEquals(true, recurrence)
        assertTrue("a second live check for the same day is refused", duplicate.isFailure)
        assertEquals(listOf("habits:60", "habit_checks:61", "habit_checks:63"), logged)
    }

    @Test
    fun versionFourAddsDayPlansAndLoggedNotesWithOneLiveNoteADay() {
        helper.createDatabase(3).apply {
            execSQL(
                "INSERT INTO projects (id, created_at, updated_at, device_id, deleted, name, slug, stage, " +
                    "description, tags, archived) VALUES ('p1', 1, 1, 'd', 0, 'Old', 'old', 'idea', '', '[]', 0)",
            )
            execSQL(
                "INSERT INTO tasks (id, created_at, updated_at, device_id, deleted, project_id, title, body_md, acceptance_md, " +
                    "related_files, status, priority, order_key, tags) VALUES ('t1', 1, 1, 'd', 0, 'p1', 'Old', '', '', '[]', " +
                    "'todo', 'medium', 'a0', '[]')",
            )
            close()
        }

        val upgraded = helper.runMigrationsAndValidate(4, listOf(MIGRATION_3_4))
        val plan = upgraded.prepare("SELECT planned_on, scheduled_at, duration_min FROM tasks WHERE id = 't1'").use { statement ->
            statement.step()
            List(3) { statement.isNull(it) }
        }
        val note = "INSERT INTO daily_notes (id, created_at, updated_at, device_id, deleted, day, body) VALUES "
        upgraded.execSQL("$note ('n1', 1, 71, 'd', 0, '2026-11-10', 'Plan')")
        val duplicate = runCatching { upgraded.execSQL("$note ('n2', 1, 72, 'd', 0, '2026-11-10', 'Twin')") }
        upgraded.execSQL("$note ('n3', 1, 73, 'd', 1, '2026-11-10', 'Gone')")
        val logged = upgraded.prepare("SELECT table_name || ':' || hlc FROM change_log ORDER BY hlc").use { statement ->
            generateSequence { if (statement.step()) statement.getText(0) else null }.toList()
        }
        upgraded.close()

        assertEquals(listOf(true, true, true), plan)
        assertTrue("a second live note for the same day is refused", duplicate.isFailure)
        assertEquals(listOf("daily_notes:71", "daily_notes:73"), logged)
    }
}
