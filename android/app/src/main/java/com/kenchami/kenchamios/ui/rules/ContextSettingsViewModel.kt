package com.kenchami.kenchamios.ui.rules

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.context.ContextPermissions
import com.kenchami.kenchamios.context.RuleRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

data class ContextSettingsState(
    val enabled: Boolean = true,
    val location: Boolean = false,
    val backgroundLocation: Boolean = false,
    val notifications: Boolean = false,
    val placeProblem: String? = null,
)

@HiltViewModel
class ContextSettingsViewModel @Inject constructor(
    private val rules: RuleRepository,
    private val coordinator: ContextCoordinator,
    private val permissions: ContextPermissions,
) : ViewModel() {
    private val permissionReads = MutableStateFlow(0)
    private val problem = MutableStateFlow<String?>(null)

    val state: StateFlow<ContextSettingsState> = combine(rules.observeEnabled(), permissionReads, problem) { enabled, _, placeProblem ->
        ContextSettingsState(enabled, permissions.location(), permissions.backgroundLocation(), permissions.notifications(), placeProblem)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), ContextSettingsState())

    /** Reads the permissions again and registers the places; call it when the screen shows and after a permission answer. */
    fun reload() {
        permissionReads.value++
        viewModelScope.launch { problem.value = coordinator.refresh() }
    }

    fun setEnabled(enabled: Boolean) {
        viewModelScope.launch {
            rules.setEnabled(enabled)
            problem.value = coordinator.refresh()
        }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
