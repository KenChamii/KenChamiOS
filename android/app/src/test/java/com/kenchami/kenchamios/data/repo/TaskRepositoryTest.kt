package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.time.LocalDate
import java.time.ZoneOffset

@RunWith(RobolectricTestRunner::class)
class TaskRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var tasks: TaskRepository
    private lateinit var projectId: String

    @Before
    fun setUp() {
        env = TestDatabase()
        tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
        projectId = runBlocking { ProjectRepository(env.database, env.stamper, env.activity).create("P") }
    }

    @After
    fun tearDown() = env.close()

    private suspend fun titles(status: TaskStatus) =
        tasks.observeTopLevel(projectId).first().filter { it.status == status }.map { it.title }

    @Test
    fun createAppendsInOrderAndRecordsActivity() = runBlocking<Unit> {
        val a = tasks.create(projectId, "A")
        tasks.create(projectId, "B")

        assertEquals(listOf("A", "B"), titles(TaskStatus.Todo))
        assertEquals(listOf("task.created"), env.database.activityDao().kindsFor(a))
    }

    @Test
    fun createKeepsTheFields() = runBlocking<Unit> {
        val id = tasks.create(projectId, " Ship ", Priority.High, dueAt = 1_700_000_000_000)

        val task = checkNotNull(tasks.get(id))
        assertEquals("Ship", task.title)
        assertEquals(Priority.High, task.priority)
        assertEquals(1_700_000_000_000, task.dueAt)
        assertEquals(TaskStatus.Todo, task.status)
    }

    @Test
    fun createRejectsABlankTitle() {
        assertThrows(IllegalArgumentException::class.java) { runBlocking { tasks.create(projectId, " ") } }
    }

    @Test
    fun setStatusPutsTheTaskLastInItsNewColumn() = runBlocking<Unit> {
        val a = tasks.create(projectId, "A")
        val b = tasks.create(projectId, "B")
        tasks.create(projectId, "C")

        tasks.setStatus(b, TaskStatus.Doing)
        tasks.setStatus(a, TaskStatus.Doing)

        assertEquals(listOf("C"), titles(TaskStatus.Todo))
        assertEquals(listOf("B", "A"), titles(TaskStatus.Doing))
        assertEquals(listOf("task.created", "task.moved"), env.database.activityDao().kindsFor(a))
    }

    @Test
    fun deleteHidesTheTask() = runBlocking<Unit> {
        val a = tasks.create(projectId, "A")
        tasks.create(projectId, "B")

        tasks.delete(a)

        assertEquals(listOf("B"), titles(TaskStatus.Todo))
        assertThrows(IllegalStateException::class.java) { runBlocking { tasks.delete(a) } }
    }

    @Test
    fun completingARepeatingTaskAddsTheNextOneOnce() = runBlocking<Unit> {
        // the test clock is 2023-11-14 in UTC
        val due = startOfDay(LocalDate.of(2023, 11, 14), ZoneOffset.UTC)
        val task = tasks.create(projectId, "Water plants", Priority.High, dueAt = due, recurrence = "daily:2")

        val nextId = checkNotNull(tasks.setStatus(task, TaskStatus.Done))

        val next = checkNotNull(tasks.get(nextId))
        assertEquals("Water plants", next.title)
        assertEquals(TaskStatus.Todo, next.status)
        assertEquals(Priority.High, next.priority)
        assertEquals("daily:2", next.recurrence)
        assertEquals(startOfDay(LocalDate.of(2023, 11, 16), ZoneOffset.UTC), next.dueAt)
        // moving within done again is not a new completion
        assertNull(tasks.setStatus(task, TaskStatus.Done))
        assertEquals(2, tasks.observeTopLevel(projectId).first().size)
    }

    @Test
    fun oneOffTasksAddNothingAndBadRulesAreRefused() = runBlocking<Unit> {
        val task = tasks.create(projectId, "Once")

        assertNull(tasks.setStatus(task, TaskStatus.Done))
        assertThrows(IllegalArgumentException::class.java) { runBlocking { tasks.create(projectId, "Bad", recurrence = "yearly:1") } }
    }

    @Test
    fun theFirstInboxTaskCreatesTheInboxWithItsFixedId() = runBlocking<Unit> {
        tasks.create(Inbox.ID, "First")
        tasks.create(Inbox.ID, "Second")

        assertEquals(1, env.count("SELECT COUNT(*) FROM projects WHERE id = '${Inbox.ID}' AND slug = 'inbox' AND name = 'Inbox'"))
        assertEquals(listOf("First", "Second"), tasks.observeTopLevel(Inbox.ID).first().map { it.title })
    }

    @Test
    fun aDeletedInboxComesBackOnTheNextCapture() = runBlocking<Unit> {
        tasks.create(Inbox.ID, "First")
        env.database.openHelper.writableDatabase.execSQL("UPDATE projects SET deleted = 1 WHERE id = '${Inbox.ID}'")

        tasks.create(Inbox.ID, "Again")

        assertEquals(1, env.count("SELECT COUNT(*) FROM projects WHERE id = '${Inbox.ID}' AND deleted = 0"))
    }

    @Test
    fun moveToProjectTakesTheSubtasksAlong() = runBlocking<Unit> {
        val captured = tasks.create(Inbox.ID, "Captured")
        env.database.openHelper.writableDatabase.execSQL(
            "INSERT INTO tasks (id, created_at, updated_at, device_id, deleted, project_id, parent_id, title, body_md, acceptance_md, " +
                "related_files, status, priority, order_key, tags) " +
                "VALUES ('sub', 1, 1, 'd', 0, '${Inbox.ID}', '$captured', 'Sub', '', '', '[]', 'todo', 'medium', 'a0', '[]')",
        )

        tasks.moveToProject(captured, projectId)

        assertEquals(listOf("Captured"), titles(TaskStatus.Todo))
        assertEquals(0, env.count("SELECT COUNT(*) FROM tasks WHERE project_id = '${Inbox.ID}'"))
        assertThrows(IllegalArgumentException::class.java) { runBlocking { tasks.moveToProject("sub", Inbox.ID) } }
        assertThrows(IllegalStateException::class.java) { runBlocking { tasks.moveToProject(captured, "missing") } }
    }

    @Test
    fun dueTasksAreOpenOnesBeforeTheLimitOldestFirst() = runBlocking<Unit> {
        val late = tasks.create(projectId, "Late", dueAt = 1_000)
        tasks.create(projectId, "Today", dueAt = 5_000)
        tasks.create(projectId, "Later", dueAt = 9_000)
        tasks.create(projectId, "No date")
        val finished = tasks.create(projectId, "Finished", dueAt = 500)
        tasks.setStatus(finished, TaskStatus.Done)

        val due = tasks.observeDue(until = 6_000).first()

        assertEquals(listOf("Late", "Today"), due.map { it.title })
        assertEquals("P", due.first().projectName)
        assertEquals(late, due.first().id)
    }
}
