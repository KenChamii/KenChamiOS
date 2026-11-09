package com.kenchami.kenchamios.ui.projects

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.EmptyState
import com.kenchami.kenchamios.ui.common.QuickAddButton
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.label
import com.kenchami.kenchamios.ui.common.listMotion
import com.kenchami.kenchamios.ui.common.timeAgo
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

@Composable
fun ProjectsScreen(onOpenProject: (String) -> Unit, onQuickAdd: () -> Unit, viewModel: ProjectsViewModel = hiltViewModel()) {
    val query by viewModel.query.collectAsStateWithLifecycle()
    val projects by viewModel.projects.collectAsStateWithLifecycle()
    AppScreen("Projects", area = Area.Projects, actions = { QuickAddButton(onQuickAdd) }) { padding ->
        ProjectsContent(query, viewModel::setQuery, projects, System.currentTimeMillis(), onOpenProject, onQuickAdd, padding)
    }
}

@Composable
fun ProjectsContent(
    query: String,
    onQueryChange: (String) -> Unit,
    projects: List<ProjectSummary>,
    now: Long,
    onOpenProject: (String) -> Unit,
    onAddIdea: () -> Unit,
    padding: PaddingValues,
) {
    val tint = Area.Projects.tint
    Column(Modifier.padding(padding)) {
        AppTextField(
            value = query,
            onValueChange = onQueryChange,
            label = "Search",
            singleLine = true,
            leadingIcon = Icons.Filled.Search,
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
        )
        when {
            projects.isNotEmpty() -> Unit
            query.isNotBlank() -> EmptyState(Icons.Filled.Search, "No project matches \"$query\".", "Clear search", { onQueryChange("") })
            else -> EmptyState(Area.Projects.icon, "No projects yet.", "Add an idea", onAddIdea, tint = tint)
        }
        LazyColumn(Modifier.testTag("projects")) {
            items(projects, key = { it.project.id }) { summary ->
                val project = summary.project
                ListItem(
                    headlineContent = { Text(project.name) },
                    supportingContent = {
                        Text(
                            "${project.stage.label} - ${summary.tasksDone}/${summary.tasksTotal} tasks - ${timeAgo(
                                summary.lastActivityAt,
                                now,
                            )}",
                        )
                    },
                    leadingContent = { TintedIcon(project.stage.icon, tint) },
                    modifier = Modifier.listMotion(this).clickable { onOpenProject(project.id) },
                )
            }
        }
    }
}
