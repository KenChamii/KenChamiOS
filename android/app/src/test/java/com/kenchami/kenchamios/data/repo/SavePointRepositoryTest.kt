package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.SavePointSource
import com.kenchami.kenchamios.data.model.Stage
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SavePointRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var projects: ProjectRepository
    private lateinit var savePoints: SavePointRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        projects = ProjectRepository(env.database, env.stamper, env.activity)
        savePoints = SavePointRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun createStoresTheNoteAtTheCurrentStage() = runBlocking<Unit> {
        val project = projects.create("P")
        projects.setStage(project, Stage.Build)

        savePoints.create(project, " stopped here ", doneMd = " - auth ", nextMd = " - form ")

        val saved = savePoints.observe(project).first().single()
        assertEquals("stopped here", saved.note)
        assertEquals("- auth", saved.doneMd)
        assertEquals("- form", saved.nextMd)
        assertEquals(Stage.Build, saved.stage)
        assertEquals(SavePointSource.Manual, saved.source)
        assertEquals(listOf("save_point.created"), env.database.activityDao().kindsFor(saved.id))
    }

    @Test
    fun listIsNewestFirst() = runBlocking<Unit> {
        val project = projects.create("P")
        savePoints.create(project, "first")
        env.clock.ms += 1_000
        savePoints.create(project, "second")

        assertEquals(listOf("second", "first"), savePoints.observe(project).first().map { it.note })
    }

    @Test
    fun createRejectsABlankNote() = runBlocking<Unit> {
        val project = projects.create("P")
        assertThrows(IllegalArgumentException::class.java) { runBlocking { savePoints.create(project, " ") } }
    }

    @Test
    fun deleteHidesTheSavePointAndRecordsActivity() = runBlocking<Unit> {
        val project = projects.create("P")
        savePoints.create(project, "one")
        val id = savePoints.observe(project).first().single().id

        savePoints.delete(id)

        assertEquals(emptyList<String>(), savePoints.observe(project).first().map { it.note })
        assertEquals(listOf("save_point.created", "save_point.deleted"), env.database.activityDao().kindsFor(id))
    }
}
