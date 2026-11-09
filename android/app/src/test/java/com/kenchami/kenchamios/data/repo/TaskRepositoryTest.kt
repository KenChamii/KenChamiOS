package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
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
class TaskRepositoryTest {
    private lateinit var env: TestDatabase
    private lateinit var tasks: TaskRepository
    private lateinit var projectId: String

    @Before
    fun setUp() {
        env = TestDatabase()
        tasks = TaskRepository(env.database, env.stamper, env.activity)
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
