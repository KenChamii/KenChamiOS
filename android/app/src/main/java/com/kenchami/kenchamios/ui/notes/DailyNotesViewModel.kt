package com.kenchami.kenchamios.ui.notes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.core.localDates
import com.kenchami.kenchamios.data.repo.DailyNoteRepository
import com.kenchami.kenchamios.sync.AppScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Clock
import java.time.LocalDate
import java.time.YearMonth
import javax.inject.Inject

private const val AUTOSAVE_MS = 800L

data class DailyNotesState(val today: LocalDate, val month: YearMonth, val selected: LocalDate, val noteDays: Set<String>)

@HiltViewModel
class DailyNotesViewModel @Inject constructor(
    private val notes: DailyNoteRepository,
    @AppScope private val appScope: CoroutineScope,
    clock: Clock,
) : ViewModel() {
    private val selected = MutableStateFlow(LocalDate.now(clock))
    private val month = MutableStateFlow(YearMonth.from(selected.value))
    private val text = MutableStateFlow<String?>(null)
    private var load: Job? = null
    private var pendingSave: Job? = null
    private var pendingEdit: Pair<LocalDate, String>? = null

    /** The selected day's note as typed, null while it loads. */
    val draft: StateFlow<String?> = text

    @OptIn(ExperimentalCoroutinesApi::class)
    val state: StateFlow<DailyNotesState?> = combine(
        localDates(clock),
        month,
        selected,
        month.flatMapLatest { notes.observeDaysWithText(it.atDay(1), it.atEndOfMonth()) },
    ) { today, shownMonth, day, noteDays -> DailyNotesState(today, shownMonth, day, noteDays) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    init {
        open(selected.value)
    }

    fun select(day: LocalDate) {
        if (day == selected.value) return
        saveNow()
        selected.value = day
        month.value = YearMonth.from(day)
        open(day)
    }

    fun showMonth(step: Long) {
        month.value = month.value.plusMonths(step)
    }

    // saves on the app scope, so leaving the screen right after typing keeps the last words
    fun edit(body: String) {
        text.value = body
        val edit = selected.value to body
        pendingEdit = edit
        pendingSave?.cancel()
        pendingSave = appScope.launch {
            delay(AUTOSAVE_MS)
            notes.save(edit.first, edit.second)
        }
    }

    private fun saveNow() {
        val edit = pendingEdit ?: return
        val job = pendingSave ?: return
        pendingEdit = null
        if (!job.isActive) return
        job.cancel()
        appScope.launch { notes.save(edit.first, edit.second) }
    }

    private fun open(day: LocalDate) {
        text.value = null
        load?.cancel()
        load = viewModelScope.launch { text.value = notes.body(day) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
