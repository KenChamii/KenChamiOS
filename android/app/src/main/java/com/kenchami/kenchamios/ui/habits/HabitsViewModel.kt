package com.kenchami.kenchamios.ui.habits

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.core.HabitSummary
import com.kenchami.kenchamios.core.dayBounds
import com.kenchami.kenchamios.data.model.TrashKind
import com.kenchami.kenchamios.data.repo.HabitRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import com.kenchami.kenchamios.ui.common.UndoBus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import java.time.LocalDate
import javax.inject.Inject

@HiltViewModel
class HabitsViewModel @Inject constructor(
    private val habits: HabitRepository,
    private val trash: TrashRepository,
    private val undo: UndoBus,
    private val clock: Clock,
) : ViewModel() {
    @OptIn(ExperimentalCoroutinesApi::class)
    val list: StateFlow<List<HabitSummary>?> = days()
        .flatMapLatest { habits.observe(it) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun add(name: String) {
        viewModelScope.launch { habits.create(name) }
    }

    fun toggleToday(id: String) {
        viewModelScope.launch { habits.toggle(id, LocalDate.now(clock)) }
    }

    fun delete(id: String) {
        viewModelScope.launch {
            habits.delete(id)
            undo.offer("Habit deleted") { trash.restore(TrashKind.Habit, id) }
        }
    }

    // today moves on at midnight while the screen stays open
    private fun days(): Flow<LocalDate> = flow {
        while (true) {
            emit(LocalDate.now(clock))
            delay(dayBounds(clock.millis(), clock.zone).end - clock.millis())
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
