package com.kenchami.kenchamios.ui.inbox

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.ui.common.UndoableTasks
import com.kenchami.kenchamios.ui.quickadd.create
import com.kenchami.kenchamios.ui.quickadd.matchProject
import com.kenchami.kenchamios.ui.quickadd.parseQuickAdd
import com.kenchami.kenchamios.ui.quickadd.toNewTask
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import java.time.LocalDate
import javax.inject.Inject

@HiltViewModel
class InboxViewModel @Inject constructor(
    private val tasks: TaskRepository,
    projects: ProjectRepository,
    private val taskActions: UndoableTasks,
    private val clock: Clock,
) : ViewModel() {
    // open tasks first, done ones stay until they are moved or deleted
    val inbox: StateFlow<List<TaskEntity>?> = tasks.observeTopLevel(Inbox.ID)
        .map { list -> list.sortedBy { it.status == TaskStatus.Done } }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    val projects: StateFlow<List<ProjectSummary>> = projects.observeSummaries(null)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), emptyList())

    /** A `#project` in the line files the task right away, everything else lands in the inbox. */
    fun capture(text: String) {
        val parsed = parseQuickAdd(text, LocalDate.now(clock))
        if (parsed.title.isBlank()) return
        val target = parsed.project?.let { matchProject(it, projects.value) }?.project?.id ?: Inbox.ID
        viewModelScope.launch { tasks.create(parsed.toNewTask(target, clock.zone)) }
    }

    fun setStatus(id: String, status: TaskStatus) {
        viewModelScope.launch { taskActions.setStatus(id, status) }
    }

    fun delete(id: String) {
        viewModelScope.launch { taskActions.delete(id) }
    }

    fun moveTo(id: String, projectId: String) {
        viewModelScope.launch { tasks.moveToProject(id, projectId) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
