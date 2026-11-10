package com.kenchami.kenchamios.ui.planner

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.core.localDates
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.repo.DailyNoteRepository
import com.kenchami.kenchamios.data.repo.PlannerRepository
import com.kenchami.kenchamios.data.repo.TaskPlan
import com.kenchami.kenchamios.ui.common.UndoableTasks
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import java.time.LocalDate
import javax.inject.Inject

@HiltViewModel
class ReviewDayViewModel @Inject constructor(
    private val planner: PlannerRepository,
    private val notes: DailyNoteRepository,
    private val taskActions: UndoableTasks,
    private val clock: Clock,
) : ViewModel() {
    @OptIn(ExperimentalCoroutinesApi::class)
    val planned: StateFlow<List<PlannedTask>?> = localDates(clock)
        .flatMapLatest { planner.observePlanned(it) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun done(task: PlannedTask) {
        viewModelScope.launch { taskActions.setStatus(task.id, TaskStatus.Done) }
    }

    fun tomorrow(task: PlannedTask) = plan(task.id, TaskPlan(LocalDate.now(clock).plusDays(1), null, task.durationMin))

    fun drop(task: PlannedTask) = plan(task.id, TaskPlan.NONE)

    fun addNote(text: String) {
        viewModelScope.launch { notes.append(LocalDate.now(clock), text) }
    }

    private fun plan(id: String, plan: TaskPlan) {
        viewModelScope.launch { planner.setPlan(id, plan) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
