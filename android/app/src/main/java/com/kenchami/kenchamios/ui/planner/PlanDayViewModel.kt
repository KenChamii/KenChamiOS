package com.kenchami.kenchamios.ui.planner

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.core.Capacity
import com.kenchami.kenchamios.core.DEFAULT_ESTIMATE_MIN
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.core.capacity
import com.kenchami.kenchamios.core.localDates
import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.db.PlannedTask
import com.kenchami.kenchamios.data.repo.PlannerRepository
import com.kenchami.kenchamios.data.repo.TaskPlan
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import java.time.LocalDate
import javax.inject.Inject

data class PlanDayState(
    val today: LocalDate,
    val bounds: DayBounds,
    val planned: List<PlannedTask>,
    val candidates: List<PlannedTask>,
    val capacity: Capacity,
)

@HiltViewModel
class PlanDayViewModel @Inject constructor(private val planner: PlannerRepository, private val clock: Clock) : ViewModel() {
    private val search = MutableStateFlow("")
    val query: StateFlow<String> = search

    @OptIn(ExperimentalCoroutinesApi::class)
    val state: StateFlow<PlanDayState?> = localDates(clock)
        .flatMapLatest { today ->
            val bounds = DayBounds(startOfDay(today, clock.zone), startOfDay(today.plusDays(1), clock.zone))
            combine(
                planner.observePlanned(today),
                search.flatMapLatest { planner.observeCandidates(today, bounds.end, it) },
                planner.observeAvailableMinutes(),
            ) { planned, candidates, available ->
                PlanDayState(today, bounds, planned, candidates, capacity(planned.map { it.durationMin }, available))
            }
        }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun search(text: String) {
        search.value = text
    }

    fun add(task: PlannedTask) = plan(task.id, TaskPlan(LocalDate.now(clock), null, task.durationMin ?: DEFAULT_ESTIMATE_MIN))

    fun setEstimate(task: PlannedTask, minutes: Int?) = plan(task.id, TaskPlan(LocalDate.now(clock), task.scheduledAt, minutes))

    // the estimate stays, so planning the task again keeps it
    fun remove(task: PlannedTask) = plan(task.id, TaskPlan(null, null, task.durationMin))

    private fun plan(id: String, plan: TaskPlan) {
        viewModelScope.launch { planner.setPlan(id, plan) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
