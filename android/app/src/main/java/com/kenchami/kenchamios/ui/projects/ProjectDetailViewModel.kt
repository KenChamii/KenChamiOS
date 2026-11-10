package com.kenchami.kenchamios.ui.projects

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.SavePointEntity
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TrashKind
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.SavePointRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import com.kenchami.kenchamios.timer.TimerController
import com.kenchami.kenchamios.timer.TimerService
import com.kenchami.kenchamios.ui.common.UndoBus
import com.kenchami.kenchamios.ui.common.UndoableTasks
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

/** The summary is null once the project was deleted. */
data class ProjectDetailState(
    val summary: ProjectSummary?,
    val tasks: List<TaskEntity>,
    val savePoints: List<SavePointEntity>,
    val focusRunning: Boolean,
)

@HiltViewModel
class ProjectDetailViewModel @Inject constructor(
    savedState: SavedStateHandle,
    private val projects: ProjectRepository,
    private val tasks: TaskRepository,
    private val taskActions: UndoableTasks,
    private val trash: TrashRepository,
    private val undo: UndoBus,
    private val savePoints: SavePointRepository,
    private val timer: TimerController,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {
    private val projectId: String = checkNotNull(savedState["id"])

    val state: StateFlow<ProjectDetailState?> = combine(
        projects.observeSummary(projectId),
        tasks.observeTopLevel(projectId),
        savePoints.observe(projectId),
        timer.state,
    ) { summary, taskList, points, running -> ProjectDetailState(summary, taskList, points, running != null) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun setStage(stage: Stage) = launch { projects.setStage(projectId, stage) }

    fun startFocus(taskId: String) = launch {
        timer.start(taskId)
        TimerService.start(context)
    }

    fun deleteProject() = launch {
        projects.delete(projectId)
        undo.offer("Project deleted") { trash.restore(TrashKind.Project, projectId) }
    }

    fun addTask(title: String) = launch { tasks.create(projectId, title) }

    fun setTaskStatus(taskId: String, status: TaskStatus) = launch { taskActions.setStatus(taskId, status) }

    fun deleteTask(taskId: String) = launch { taskActions.delete(taskId) }

    fun addSavePoint(note: String, done: String, next: String) = launch { savePoints.create(projectId, note, done, next) }

    fun deleteSavePoint(id: String) = launch {
        savePoints.delete(id)
        undo.offer("Save point deleted") { trash.restore(TrashKind.SavePoint, id) }
    }

    private fun launch(block: suspend () -> Unit) {
        viewModelScope.launch { block() }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
