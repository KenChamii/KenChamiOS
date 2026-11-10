package com.kenchami.kenchamios.ui.prompts

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import com.kenchami.kenchamios.data.model.TrashKind
import com.kenchami.kenchamios.data.repo.PromptRepository
import com.kenchami.kenchamios.data.repo.TrashRepository
import com.kenchami.kenchamios.ui.common.UndoBus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class PromptsViewModel @Inject constructor(
    private val repository: PromptRepository,
    private val trash: TrashRepository,
    private val undo: UndoBus,
) : ViewModel() {
    val prompts: StateFlow<List<PromptEntity>> = repository.observeAll()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), emptyList())

    /** The text to put on the clipboard; counts one use. */
    suspend fun copyText(id: String): String = repository.copy(id)

    fun setFavorite(id: String, favorite: Boolean) {
        viewModelScope.launch { repository.setFavorite(id, favorite) }
    }

    fun delete(id: String) {
        viewModelScope.launch {
            repository.delete(id)
            undo.offer("Prompt deleted") { trash.restore(TrashKind.Prompt, id) }
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
