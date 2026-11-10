package com.kenchami.kenchamios.ui.habits

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.core.HabitSummary
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

@Composable
fun HabitsScreen(onBack: () -> Unit, viewModel: HabitsViewModel = hiltViewModel()) {
    val habits by viewModel.list.collectAsStateWithLifecycle()
    var name by rememberSaveable { mutableStateOf("") }
    val nameFocus = remember { FocusRequester() }

    AppScreen("Habits", onBack = onBack, area = Area.Workflow) { padding ->
        LazyColumn(contentPadding = padding, modifier = Modifier.testTag("habits")) {
            item {
                Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    AppTextField(name, { name = it }, "New habit", Modifier.weight(1f).focusRequester(nameFocus), singleLine = true)
                    IconButton(
                        onClick = {
                            viewModel.add(name)
                            name = ""
                        },
                        enabled = name.isNotBlank(),
                    ) { Icon(Icons.Filled.Add, contentDescription = "Add habit") }
                }
            }
            if (habits?.isEmpty() == true) {
                item(key = "habits-empty") {
                    EmptyState(Icons.Filled.Star, "No habits yet.", "Start one", { nameFocus.requestFocus() }, tint = Area.Workflow.tint)
                }
            }
            items(habits.orEmpty(), key = { it.id }) { habit ->
                HabitRow(habit, { viewModel.toggleToday(habit.id) }, { viewModel.delete(habit.id) }, Modifier.listMotion(this))
            }
        }
    }
}

@Composable
private fun HabitRow(habit: HabitSummary, onToggle: () -> Unit, onDelete: () -> Unit, modifier: Modifier = Modifier) {
    ListItem(
        modifier = modifier,
        headlineContent = { Text(habit.name) },
        supportingContent = {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(if (habit.streak == 1) "1 day streak" else "${habit.streak} day streak")
                LastWeek(habit.recent)
            }
        },
        leadingContent = { Checkbox(checked = habit.checkedToday, onCheckedChange = { onToggle() }) },
        trailingContent = {
            IconButton(onClick = onDelete) { Icon(Icons.Filled.Delete, contentDescription = "Delete habit") }
        },
    )
}

// one dot per day, oldest first, today last
@Composable
private fun LastWeek(recent: List<Boolean>) {
    val done = Area.Workflow.tint.content
    val open = MaterialTheme.colorScheme.surfaceContainerHigh
    Row(
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        modifier = Modifier.semantics { contentDescription = "${recent.count { it }} of the last ${recent.size} days" },
    ) {
        recent.forEach { checked ->
            Box(Modifier.size(10.dp).background(if (checked) done else open, CircleShape))
        }
    }
}
