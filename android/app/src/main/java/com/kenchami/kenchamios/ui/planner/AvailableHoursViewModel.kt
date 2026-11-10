package com.kenchami.kenchamios.ui.planner

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.repo.PlannerRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AvailableHoursViewModel @Inject constructor(private val planner: PlannerRepository) : ViewModel() {
    val minutes: StateFlow<Int?> = planner.observeAvailableMinutes()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun save(minutes: Int) {
        viewModelScope.launch { planner.setAvailableMinutes(minutes) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
