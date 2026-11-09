package com.kenchami.kenchamios.ui.media

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.TrashKind
import com.kenchami.kenchamios.data.repo.EntryInput
import com.kenchami.kenchamios.data.repo.MediaRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import com.kenchami.kenchamios.ui.common.UndoBus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class EntryDetailViewModel @Inject constructor(
    savedState: SavedStateHandle,
    private val repository: MediaRepository,
    private val trash: TrashRepository,
    private val undo: UndoBus,
) : ViewModel() {
    private val entryId: String = checkNotNull(savedState["entryId"])

    /** Null while loading and after the entry was deleted. */
    val entry: StateFlow<LibraryEntry?> = repository.observeEntry(entryId)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    fun save(input: EntryInput) {
        viewModelScope.launch { repository.update(entryId, input) }
    }

    fun addOne() {
        viewModelScope.launch { repository.increment(entryId) }
    }

    fun subtractOne() {
        viewModelScope.launch { repository.decrement(entryId) }
    }

    fun delete() {
        viewModelScope.launch {
            repository.delete(entryId)
            undo.offer("Removed from the library") { trash.restore(TrashKind.Media, entryId) }
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
