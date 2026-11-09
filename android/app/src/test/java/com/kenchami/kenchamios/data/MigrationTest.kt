package com.kenchami.kenchamios.data

import android.content.Context
import androidx.room.testing.MigrationTestHelper
import androidx.sqlite.driver.AndroidSQLiteDriver
import androidx.sqlite.execSQL
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.AppDatabase_Impl
import org.junit.Assert.assertEquals
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
}
