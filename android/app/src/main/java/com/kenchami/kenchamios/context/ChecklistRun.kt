package com.kenchami.kenchamios.context

data class ChecklistItem(val text: String, val checked: Boolean)

/** One round of a checklist rule; a new trigger starts a new run with nothing ticked. */
data class ChecklistRun(val ruleId: String, val ruleName: String, val items: List<ChecklistItem>) {
    val complete: Boolean get() = items.all { it.checked }

    fun toggled(index: Int): ChecklistRun =
        copy(items = items.mapIndexed { i, item -> if (i == index) item.copy(checked = !item.checked) else item })

    companion object {
        fun start(rule: Rule, action: RuleAction.Checklist) =
            ChecklistRun(rule.id, rule.name, action.items.map { ChecklistItem(it, checked = false) })
    }
}
