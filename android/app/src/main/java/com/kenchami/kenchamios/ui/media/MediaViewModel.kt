package com.kenchami.kenchamios.ui.media

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.LibraryEntry
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.repo.MediaRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

data class MediaFilters(val type: MediaType? = null, val status: MediaStatus? = null)

data class MediaState(val filters: MediaFilters, val entries: List<LibraryEntry>)

@HiltViewModel
class MediaViewModel @Inject constructor(private val repository: MediaRepository) : ViewModel() {
    private val filters = MutableStateFlow(MediaFilters())

    val state: StateFlow<MediaState> = combine(filters, repository.observeLibrary()) { current, library ->
        MediaState(
            current,
            library.filter {
                (current.type == null || it.mediaType == current.type) &&
                    (current.status == null || it.status == current.status)
            },
        )
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), MediaState(MediaFilters(), emptyList()))

    fun setType(type: MediaType?) {
        filters.value = filters.value.copy(type = type)
    }

    fun setStatus(status: MediaStatus?) {
        filters.value = filters.value.copy(status = status)
    }

    fun addOne(entryId: String) {
        viewModelScope.launch { repository.increment(entryId) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
