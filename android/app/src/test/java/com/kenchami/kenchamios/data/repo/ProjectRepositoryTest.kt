package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class ProjectRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var projects: ProjectRepository
    private lateinit var tasks: TaskRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        projects = ProjectRepository(env.database, env.stamper, env.activity)
        tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
    }

    @After
    fun tearDown() = env.close()

    @Test
    fun createStoresTheProjectAndRecordsActivity() = runBlocking<Unit> {
        val id = projects.create("  My App ", " about ")

        val summary = checkNotNull(projects.observeSummary(id).first())
        assertEquals("My App", summary.project.name)
        assertEquals("my-app", summary.project.slug)
        assertEquals(Stage.Idea, summary.project.stage)
        assertEquals("about", summary.project.description)
        assertEquals(env.stamper.deviceId(), summary.project.deviceId)
        assertEquals(listOf("project.created"), env.database.activityDao().kindsFor(id))
    }

    @Test
    fun theInboxIsHiddenFromTheListAndCannotBeChanged() = runBlocking<Unit> {
        projects.create("Alpha")
        tasks.create(Inbox.ID, "Captured")

        assertEquals(listOf("Alpha"), projects.observeSummaries(null).first().map { it.project.name })
        assertThrows(IllegalArgumentException::class.java) { runBlocking { projects.delete(Inbox.ID) } }
        assertThrows(IllegalArgumentException::class.java) { runBlocking { projects.setStage(Inbox.ID, Stage.Done) } }
    }

    @Test
    fun createRejectsABlankName() {
        assertThrows(IllegalArgumentException::class.java) { runBlocking { projects.create("   ") } }
    }

    @Test
    fun slugsStayUnique() = runBlocking<Unit> {
        val first = projects.create("Same")
        val second = projects.create("Same")

        assertEquals("same", projects.observeSummary(first).first()?.project?.slug)
        assertEquals("same-2", projects.observeSummary(second).first()?.project?.slug)
    }

    @Test
    fun updatedAtGrowsWithEveryWrite() = runBlocking<Unit> {
        val id = projects.create("P")
        val before = checkNotNull(projects.observeSummary(id).first()).project.updatedAt

        projects.setStage(id, Stage.Build)

        val after = checkNotNull(projects.observeSummary(id).first()).project.updatedAt
        assertTrue(after > before)
    }

    @Test
    fun movingToDoneSetsFinishedAtAndMovingBackClearsIt() = runBlocking<Unit> {
        val id = projects.create("P")

        projects.setStage(id, Stage.Done)
        assertEquals(env.clock.ms, projects.observeSummary(id).first()?.project?.finishedAt)

        projects.setStage(id, Stage.Review)
        assertNull(projects.observeSummary(id).first()?.project?.finishedAt)
        val kinds = env.database.activityDao().kindsFor(id)
        assertEquals(listOf("project.created", "project.stage_changed", "project.stage_changed"), kinds)
    }

    @Test
    fun searchMatchesNameAndDescription() = runBlocking<Unit> {
        projects.create("Alpha", "first")
        projects.create("Beta", "has alpha inside")
        projects.create("Gamma")

        assertEquals(2, projects.observeSummaries("alp").first().size)
        assertEquals(3, projects.observeSummaries("  ").first().size)
    }

    @Test
    fun summaryCountsTopLevelTasksAndUsesTheLatestActivity() = runBlocking<Unit> {
        val id = projects.create("P")
        val done = tasks.create(id, "A")
        tasks.create(id, "B")
        env.clock.ms += 5_000
        tasks.setStatus(done, TaskStatus.Done)

        val summary = checkNotNull(projects.observeSummary(id).first())
        assertEquals(2, summary.tasksTotal)
        assertEquals(1, summary.tasksDone)
        assertEquals(env.clock.ms, summary.lastActivityAt)
    }

    @Test
    fun deleteHidesTheProjectAndEverythingInIt() = runBlocking<Unit> {
        val id = projects.create("P")
        val task = tasks.create(id, "A")
        env.database.openHelper.writableDatabase.execSQL(
            "INSERT INTO checklist_items (id, created_at, updated_at, device_id, deleted, task_id, text, done, order_key) " +
                "VALUES ('c1', 1, 1, 'd', 0, '$task', 'step', 0, 'V')",
        )
        SavePointRepository(env.database, env.stamper, env.activity).create(id, "stop")
        PromptRepository(env.database, env.stamper, env.activity).create("Review", "body", projectId = id)

        projects.delete(id)

        assertNull(projects.observeSummary(id).first())
        for (table in listOf("projects", "tasks", "checklist_items", "save_points", "prompts")) {
            assertEquals(table, 0, env.count("SELECT COUNT(*) FROM $table WHERE deleted = 0"))
        }
        assertTrue("project.deleted" in env.database.activityDao().kindsFor(id))
    }
}
