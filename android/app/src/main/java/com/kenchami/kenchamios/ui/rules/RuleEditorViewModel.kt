package com.kenchami.kenchamios.ui.rules

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.context.RuleRepository
import com.kenchami.kenchamios.context.RuleValidation
import com.kenchami.kenchamios.context.Trigger
import com.kenchami.kenchamios.data.Stamper
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

const val NEW_RULE = "new"
const val LEAVING_HOME_PRESET = "leaving-home"

@HiltViewModel
class RuleEditorViewModel @Inject constructor(
    savedState: SavedStateHandle,
    private val rules: RuleRepository,
    private val coordinator: ContextCoordinator,
    private val stamper: Stamper,
) : ViewModel() {
    private val routeId: String = checkNotNull(savedState["id"])
    val isNew = routeId == NEW_RULE || routeId == LEAVING_HOME_PRESET

    private val _form = MutableStateFlow<RuleForm?>(null)
    val form: StateFlow<RuleForm?> = _form

    private val _message = MutableStateFlow<String?>(null)
    val message: StateFlow<String?> = _message

    init {
        viewModelScope.launch {
            _form.value = when (routeId) {
                NEW_RULE -> RuleForm()
                LEAVING_HOME_PRESET -> RuleForm.leavingHome()
                else -> rules.get(routeId)?.let(RuleForm::of) ?: RuleForm()
            }
        }
    }

    fun edit(change: (RuleForm) -> RuleForm) {
        _form.update { current -> current?.let(change) }
        _message.value = null
    }

    fun save(onDone: () -> Unit) {
        val form = _form.value ?: return
        viewModelScope.launch {
            val id = if (isNew) stamper.newId() else routeId
            when (val result = form.toRule(id)) {
                is FormResult.Invalid -> _message.value = result.message
                is FormResult.Valid -> {
                    val rule = result.rule
                    val others = rules.rules().count { it.id != id && it.enabled && it.trigger is Trigger.Place }
                    val problem = RuleValidation.check(rule, others)
                    if (problem != null) {
                        _message.value = problem
                    } else {
                        rules.save(rule)
                        coordinator.refresh()
                        onDone()
                    }
                }
            }
        }
    }

    fun delete(onDone: () -> Unit) {
        viewModelScope.launch {
            rules.delete(routeId)
            coordinator.refresh()
            onDone()
        }
    }
}
