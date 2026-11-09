package com.kenchami.kenchamios.ui.projects

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.repo.ProjectRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class ProjectsViewModel @Inject constructor(repository: ProjectRepository) : ViewModel() {
    private val queryFlow = MutableStateFlow("")
    val query: StateFlow<String> = queryFlow

    @OptIn(ExperimentalCoroutinesApi::class)
    val projects: StateFlow<List<ProjectSummary>> = queryFlow
        .flatMapLatest { repository.observeSummaries(it) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), emptyList())

    fun setQuery(value: String) {
        queryFlow.value = value
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
