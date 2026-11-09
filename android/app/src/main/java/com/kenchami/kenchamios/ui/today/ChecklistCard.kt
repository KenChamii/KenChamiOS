package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.context.ChecklistRun
import com.kenchami.kenchamios.ui.common.TonalCard

class ChecklistActions(val onToggle: (ruleId: String, index: Int) -> Unit, val onDismiss: (ruleId: String) -> Unit)

/** A checklist that a rule started; ticking the last item removes it. */
@Composable
fun ChecklistCard(run: ChecklistRun, actions: ChecklistActions) {
    TonalCard(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp).testTag("checklist-${run.ruleId}")) {
        Column(Modifier.padding(vertical = 8.dp)) {
            Text(run.ruleName, style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(horizontal = 16.dp))
            run.items.forEachIndexed { index, item ->
                ListItem(
                    colors = ListItemDefaults.colors(containerColor = Color.Transparent),
                    headlineContent = { Text(item.text) },
                    leadingContent = { Checkbox(checked = item.checked, onCheckedChange = { actions.onToggle(run.ruleId, index) }) },
                    modifier = Modifier.clickable { actions.onToggle(run.ruleId, index) },
                )
            }
            TextButton(onClick = { actions.onDismiss(run.ruleId) }, modifier = Modifier.padding(horizontal = 8.dp)) { Text("Dismiss") }
        }
    }
}
