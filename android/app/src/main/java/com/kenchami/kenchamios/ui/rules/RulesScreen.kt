package com.kenchami.kenchamios.ui.rules

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.context.Rule
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.EmptyText

@Composable
fun RulesScreen(
    onBack: () -> Unit,
    onOpenRule: (String) -> Unit,
    onOpenSettings: () -> Unit,
    viewModel: RulesViewModel = hiltViewModel(),
) {
    val rules by viewModel.list.collectAsStateWithLifecycle()
    val message by viewModel.message.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(message) {
        message?.let {
            snackbar.showSnackbar(it)
            viewModel.messageShown()
        }
    }
    AppScreen("Rules", onBack = onBack, snackbar = snackbar, actions = {
        IconButton(onClick = onOpenSettings) { Icon(Icons.Filled.Settings, contentDescription = "Rule settings") }
        IconButton(onClick = { onOpenRule(NEW_RULE) }) { Icon(Icons.Filled.Add, contentDescription = "Add rule") }
    }) { padding ->
        rules?.let { RulesContent(it, viewModel::setEnabled, onOpenRule, padding) }
    }
}

@Composable
fun RulesContent(rules: List<Rule>, onToggle: (Rule, Boolean) -> Unit, onOpenRule: (String) -> Unit, padding: PaddingValues) {
    LazyColumn(contentPadding = padding, modifier = Modifier.testTag("rules")) {
        if (rules.isEmpty()) item { EmptyText("No rules yet. Add one, or start from the Leaving home checklist.") }
        items(rules, key = { it.id }) { rule ->
            ListItem(
                headlineContent = { Text(rule.name) },
                supportingContent = { Text(ruleSummary(rule)) },
                trailingContent = {
                    Switch(checked = rule.enabled, onCheckedChange = { onToggle(rule, it) })
                },
                modifier = Modifier.clickable { onOpenRule(rule.id) },
            )
        }
        item {
            TextButton(onClick = { onOpenRule(LEAVING_HOME_PRESET) }, modifier = Modifier.padding(horizontal = 8.dp)) {
                Text("Add the Leaving home checklist")
            }
        }
    }
}
