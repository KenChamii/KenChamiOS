package com.kenchami.kenchamios.context

import com.kenchami.kenchamios.data.LocalSettings
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

internal const val RUN = "context.run."

/** The checklists that are waiting to be ticked, one per rule. */
@Singleton
class ChecklistRunRepository @Inject constructor(private val settings: LocalSettings) {
    fun observe(): Flow<List<ChecklistRun>> = settings.observePrefix(RUN).map { rows ->
        rows.sortedBy { it.key }.mapNotNull { RuleCodec.decodeRun(it.key.removePrefix(RUN), it.valueJson) }
    }

    /** Replaces the run of the rule, so every trigger starts with empty boxes. */
    suspend fun start(run: ChecklistRun) = settings.put(RUN + run.ruleId, RuleCodec.encode(run))

    /** A run with every item ticked is done and disappears. */
    suspend fun toggle(ruleId: String, index: Int) {
        val run = settings.get(RUN + ruleId)?.let { RuleCodec.decodeRun(ruleId, it) } ?: return
        val updated = run.toggled(index)
        if (updated.complete) dismiss(ruleId) else settings.put(RUN + ruleId, RuleCodec.encode(updated))
    }

    suspend fun dismiss(ruleId: String) = settings.delete(RUN + ruleId)
}
