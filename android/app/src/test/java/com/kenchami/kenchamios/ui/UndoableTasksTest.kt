package com.kenchami.kenchamios.ui

import com.kenchami.kenchamios.data.TestDatabase
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import com.kenchami.kenchamios.ui.common.UndoBus
import com.kenchami.kenchamios.ui.common.UndoOffer
import com.kenchami.kenchamios.ui.common.UndoableTasks
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class UndoableTasksTest {
    private val env = TestDatabase()
    private val tasks = TaskRepository(env.database, env.stamper, env.activity, env.clock)
    private val bus = UndoBus()
    private val actions = UndoableTasks(tasks, TrashRepository(env.database, env.stamper, env.activity), bus)

    @After
    fun close() = env.close()

    private suspend fun offerOf(action: suspend () -> Unit): UndoOffer = coroutineScope {
        val offer = async(start = CoroutineStart.UNDISPATCHED) { bus.offers.first() }
        action()
        offer.await()
    }

    @Test
    fun undoingACompletionReopensTheTaskAndDropsTheNextOne() = runBlocking<Unit> {
        val project = ProjectRepository(env.database, env.stamper, env.activity).create("P")
        val task = tasks.create(project, "Water", Priority.Low, recurrence = "daily:1")

        val offer = offerOf { actions.setStatus(task, TaskStatus.Done) }
        assertEquals("Task completed, the next one is added", offer.message)
        offer.undo()

        val live = tasks.observeTopLevel(project).first()
        assertEquals(listOf(task to TaskStatus.Todo), live.map { it.id to it.status })
    }

    @Test
    fun undoingADeleteRestoresTheTask() = runBlocking<Unit> {
        val project = ProjectRepository(env.database, env.stamper, env.activity).create("P")
        val task = tasks.create(project, "Keep")

        offerOf { actions.delete(task) }.undo()

        assertEquals("Keep", tasks.get(task)?.title)
    }
}
