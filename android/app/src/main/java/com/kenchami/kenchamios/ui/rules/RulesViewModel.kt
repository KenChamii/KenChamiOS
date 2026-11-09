package com.kenchami.kenchamios.ui.rules

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.context.Rule
import com.kenchami.kenchamios.context.RuleRepository
import com.kenchami.kenchamios.context.RuleValidation
import com.kenchami.kenchamios.context.Trigger
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class RulesViewModel @Inject constructor(private val rules: RuleRepository, private val coordinator: ContextCoordinator) : ViewModel() {
    val list: StateFlow<List<Rule>?> = rules.observeRules()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    private val _message = MutableStateFlow<String?>(null)
    val message: StateFlow<String?> = _message

    fun setEnabled(rule: Rule, enabled: Boolean) {
        viewModelScope.launch {
            val updated = rule.copy(enabled = enabled)
            val error = RuleValidation.check(updated, rules.rules().count { it.id != rule.id && it.enabled && it.trigger is Trigger.Place })
            if (error != null) {
                _message.value = error
                return@launch
            }
            rules.save(updated)
            coordinator.refresh()
        }
    }

    fun messageShown() {
        _message.value = null
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
