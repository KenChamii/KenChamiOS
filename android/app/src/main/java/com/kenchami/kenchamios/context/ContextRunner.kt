package com.kenchami.kenchamios.context

import java.time.Clock
import javax.inject.Inject
import javax.inject.Singleton

private const val MAX_TITLES = 5

/** Runs what an event fires: records it, then shows a notification or starts a checklist. */
@Singleton
class ContextRunner @Inject constructor(
    private val rules: RuleRepository,
    private val runs: ChecklistRunRepository,
    private val dueTasks: DueTasks,
    private val notifier: RuleNotifier,
    private val clock: Clock,
) {
    suspend fun handle(event: ContextEvent) {
        if (!rules.enabled()) return
        val now = clock.millis()
        for (rule in RuleEngine.evaluate(event, rules.rules(), rules.lastFired(), now, clock.zone)) {
            rules.markFired(rule.id, now)
            perform(rule)
        }
    }

    /** The first reading is only remembered; a change after that is an event. */
    suspend fun onWifiState(connected: Boolean) {
        val before = rules.wifiConnected()
        if (before == connected) return
        rules.setWifiConnected(connected)
        if (before != null) handle(ContextEvent.WifiChanged(if (connected) WifiTransition.Connect else WifiTransition.Disconnect))
    }

    private suspend fun perform(rule: Rule) {
        when (val action = rule.action) {
            RuleAction.ShowTasks -> showTasks(rule)
            is RuleAction.Remind -> notifier.post(rule.id, rule.name, action.text)
            is RuleAction.Checklist -> {
                runs.start(ChecklistRun.start(rule, action))
                notifier.post(rule.id, rule.name, "Check ${action.items.size} things before you go.")
            }
        }
    }

    private suspend fun showTasks(rule: Rule) {
        val titles = dueTasks.titles()
        if (titles.isEmpty()) return
        val more = titles.size - MAX_TITLES
        val text = titles.take(MAX_TITLES).joinToString(", ") + if (more > 0) " and $more more" else ""
        notifier.post(rule.id, rule.name, text)
    }
}
