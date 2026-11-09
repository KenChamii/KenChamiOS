package com.kenchami.kenchamios.ui.today

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.core.DayBounds
import com.kenchami.kenchamios.core.dayBounds
import com.kenchami.kenchamios.data.db.DueTask
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.db.entity.ActivityEntity
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.repo.ActivityRepository
import com.kenchami.kenchamios.data.repo.MediaRepository
import com.kenchami.kenchamios.data.repo.ProjectRepository
import com.kenchami.kenchamios.data.repo.TaskRepository
import com.kenchami.kenchamios.data.repo.continueItems
import com.kenchami.kenchamios.timer.TimerController
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import javax.inject.Inject

private const val ACTIVITY_LIMIT = 15

data class TodayState(
    val bounds: DayBounds,
    val now: Long,
    val due: List<DueTask>,
    val continueItems: List<LibraryEntry>,
    val projects: List<ProjectSummary>,
    val activity: List<ActivityEntity>,
)

@HiltViewModel
class TodayViewModel @Inject constructor(
    private val tasks: TaskRepository,
    private val media: MediaRepository,
    projects: ProjectRepository,
    private val timer: TimerController,
    activity: ActivityRepository,
    private val clock: Clock,
) : ViewModel() {
    @OptIn(ExperimentalCoroutinesApi::class)
    val state: StateFlow<TodayState?> = days()
        .flatMapLatest { bounds ->
            combine(
                tasks.observeDue(bounds.end),
                media.observeLibrary().map(::continueItems),
                projects.observeSummaries(null).map { list -> list.filter { it.project.stage != Stage.Done } },
                activity.observeRecent(ACTIVITY_LIMIT),
            ) { due, continuing, active, recent ->
                TodayState(bounds, clock.millis(), due, continuing, active, recent)
            }
        }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun addOne(entryId: String) {
        viewModelScope.launch { media.increment(entryId) }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    val focus: StateFlow<FocusCard?> = timer.state
        .mapLatest { running -> running?.let { FocusCard(it, tasks.get(it.taskId)?.title.orEmpty()) } }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun pauseTimer() {
        viewModelScope.launch { timer.pause() }
    }

    fun resumeTimer() {
        viewModelScope.launch { timer.resume() }
    }

    fun stopTimer() {
        viewModelScope.launch { timer.stop() }
    }

    fun finishTask(id: String) {
        viewModelScope.launch { tasks.setStatus(id, TaskStatus.Done) }
    }

    // a new boundary at midnight, so "due today" moves on while the app stays open
    private fun days(): Flow<DayBounds> = flow {
        while (true) {
            val bounds = dayBounds(clock.millis(), clock.zone)
            emit(bounds)
            delay(bounds.end - clock.millis())
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
