package com.kenchami.kenchamios.ui.quickadd

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.repo.MediaRepository
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.PromptRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import javax.inject.Inject

@HiltViewModel
class QuickAddViewModel @Inject constructor(
    private val tasks: TaskRepository,
    private val projects: ProjectRepository,
    private val prompts: PromptRepository,
    private val media: MediaRepository,
    val clock: Clock,
) : ViewModel() {
    val projectChoices: StateFlow<List<ProjectSummary>> = projects.observeSummaries(null)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), emptyList())

    fun addTask(projectId: String, title: String, priority: Priority, dueAt: Long?) {
        viewModelScope.launch { tasks.create(projectId, title, priority, dueAt) }
    }

    fun addIdea(name: String, description: String) {
        viewModelScope.launch { projects.create(name, description) }
    }

    fun addPrompt(title: String, body: String, projectId: String?) {
        viewModelScope.launch { prompts.create(title, body, projectId = projectId) }
    }

    fun addMedia(type: MediaType, title: String, status: MediaStatus, total: Int?) {
        viewModelScope.launch { media.add(type, title, status, total) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
