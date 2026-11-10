package com.kenchami.kenchamios.ui.common

import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TrashKind
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import javax.inject.Inject

/** Task changes every task list makes, with the undo the snackbar offers. */
class UndoableTasks @Inject constructor(private val tasks: TaskRepository, private val trash: TrashRepository, private val undo: UndoBus) {
    /** Completing offers an undo that reopens the task and drops the next occurrence it added. */
    suspend fun setStatus(id: String, status: TaskStatus) {
        val before = tasks.get(id)?.status ?: return
        val next = tasks.setStatus(id, status)
        if (status != TaskStatus.Done || before == TaskStatus.Done) return
        undo.offer(if (next != null) "Task completed, the next one is added" else "Task completed") {
            tasks.setStatus(id, before)
            next?.let { tasks.delete(it) }
        }
    }

    suspend fun delete(id: String) {
        tasks.delete(id)
        undo.offer("Task deleted") { trash.restore(TrashKind.Task, id) }
    }
}
