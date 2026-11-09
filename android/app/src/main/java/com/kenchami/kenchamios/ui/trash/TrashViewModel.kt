package com.kenchami.kenchamios.ui.trash

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.TrashRow
import com.kenchami.kenchamios.data.repo.TrashRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class TrashViewModel @Inject constructor(private val trash: TrashRepository) : ViewModel() {
    val rows: StateFlow<List<TrashRow>?> = trash.observe()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    /** Throws when the row is gone or a live copy of it exists. */
    suspend fun restore(row: TrashRow) = trash.restore(row.kind, row.id)

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
