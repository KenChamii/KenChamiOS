package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.TrashKind
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDate

@RunWith(RobolectricTestRunner::class)
class TrashRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var projects: ProjectRepository
    private lateinit var tasks: TaskRepository
    private lateinit var trash: TrashRepository

    @Before
    fun setUp() {
        env = TestDatabase()
        projects = ProjectRepository(env.database, env.stamper, env.activity)
        tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
        trash = TrashRepository(env.database, env.stamper, env.activity)
    }

    @After
    fun tearDown() = env.close()

    private suspend fun listed() = trash.observe().first().map { it.kind to it.id }

    @Test
    fun aTaskComesBackButNotWhatWasDeletedBeforeIt() = runBlocking<Unit> {
        val project = projects.create("P")
        val early = tasks.create(project, "Deleted earlier")
        val task = tasks.create(project, "Task")
        tasks.delete(early)
        tasks.delete(task)
        assertEquals(setOf(TrashKind.Task to task, TrashKind.Task to early), listed().toSet())

        trash.restore(TrashKind.Task, task)

        assertEquals("Task", tasks.get(task)?.title)
        assertEquals(null, tasks.get(early))
        assertEquals(listOf(TrashKind.Task to early), listed())
        assertEquals(listOf("task.created", "task.deleted", "task.restored"), env.database.activityDao().kindsFor(task))
    }

    @Test
    fun aProjectComesBackWithEverythingItsDeleteTook() = runBlocking<Unit> {
        val project = projects.create("P")
        val task = tasks.create(project, "Task")
        PromptRepository(env.database, env.stamper, env.activity).create("Prompt", "body", projectId = project)
        projects.delete(project)
        assertEquals(listOf(TrashKind.Project to project), listed())
        assertThrows(IllegalStateException::class.java) { runBlocking { trash.restore(TrashKind.Task, task) } }

        trash.restore(TrashKind.Project, project)

        assertEquals("P", projects.observeSummary(project).first()?.project?.name)
        assertEquals("Task", tasks.get(task)?.title)
        assertEquals(1, env.count("SELECT COUNT(*) FROM prompts WHERE deleted = 0"))
        assertTrue(listed().isEmpty())
        assertThrows(IllegalStateException::class.java) { runBlocking { trash.restore(TrashKind.Project, project) } }
    }

    @Test
    fun mediaAndHabitsComeBack() = runBlocking<Unit> {
        val media = MediaRepository(env.database, env.stamper, env.activity)
        val entry = media.add(MediaType.Anime, "Frieren", MediaStatus.Planned, null)
        media.delete(entry)
        val habits = HabitRepository(env.database, env.stamper, env.activity)
        val habit = habits.create("Read")
        habits.toggle(habit, LocalDate.of(2026, 11, 10))
        habits.delete(habit)

        trash.restore(TrashKind.Media, entry)
        trash.restore(TrashKind.Habit, habit)

        assertEquals("Frieren", media.observeEntry(entry).first()?.title)
        assertEquals(1, habits.observe(LocalDate.of(2026, 11, 10)).first().single().streak)
    }

    @Test
    fun restoringBumpsTheClockSoSyncSendsIt() = runBlocking<Unit> {
        val task = tasks.create(projects.create("P"), "Task")
        tasks.delete(task)
        val deletedAt = stampOf(task)

        trash.restore(TrashKind.Task, task)

        assertTrue(stampOf(task) > deletedAt)
        assertEquals(1, env.count("SELECT COUNT(*) FROM change_log l JOIN tasks t ON t.id = l.row_id AND l.hlc = t.updated_at"))
    }

    @Test
    fun theInboxIsNeverListed() = runBlocking<Unit> {
        tasks.create(Inbox.ID, "Capture")
        env.database.openHelper.writableDatabase.execSQL("UPDATE projects SET deleted = 1 WHERE id = '${Inbox.ID}'")

        assertTrue(listed().isEmpty())
    }

    private fun stampOf(id: String): Long =
        env.database.openHelper.writableDatabase.query("SELECT updated_at FROM tasks WHERE id = '$id'").use {
            it.moveToFirst()
            it.getLong(0)
        }
}
