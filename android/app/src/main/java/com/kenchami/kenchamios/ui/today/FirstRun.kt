package com.kenchami.kenchamios.ui.today

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.ui.common.TintedIcon
import com.kenchami.kenchamios.ui.common.TonalCard
import com.kenchami.kenchamios.ui.common.icon
import com.kenchami.kenchamios.ui.common.neutralTint
import com.kenchami.kenchamios.ui.theme.Area
import com.kenchami.kenchamios.ui.theme.tint

enum class SetupStep(val label: String) {
    CreateProject("Create your first project"),
    AddTask("Add a task to it"),
    AddMedia("Add a show, game or album"),
    PairDesktop("Pair with the desktop app"),
}

/** The first-run checklist: only while there is no project and no media, and pairing drops out once it is done. */
fun setupSteps(hasProjects: Boolean, hasMedia: Boolean, paired: Boolean): List<SetupStep> = when {
    hasProjects || hasMedia -> emptyList()
    paired -> SetupStep.entries - SetupStep.PairDesktop
    else -> SetupStep.entries
}

@Composable
fun SetupCard(steps: List<SetupStep>, onStep: (SetupStep) -> Unit) {
    TonalCard(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp).testTag("setup")) {
        Column(Modifier.padding(vertical = 12.dp)) {
            Text("Get started", style = MaterialTheme.typography.titleLarge, modifier = Modifier.padding(horizontal = 20.dp))
            steps.forEach { step ->
                ListItem(
                    colors = ListItemDefaults.colors(containerColor = Color.Transparent),
                    headlineContent = { Text(step.label) },
                    leadingContent = {
                        when (step) {
                            SetupStep.CreateProject -> TintedIcon(Area.Projects.icon, Area.Projects.tint)
                            SetupStep.AddTask -> TintedIcon(Icons.Filled.CheckCircle, Area.Projects.tint)
                            SetupStep.AddMedia -> TintedIcon(Area.Media.icon, Area.Media.tint)
                            SetupStep.PairDesktop -> TintedIcon(Icons.Filled.Refresh, neutralTint)
                        }
                    },
                    trailingContent = { Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, contentDescription = null) },
                    modifier = Modifier.clickable { onStep(step) },
                )
            }
        }
    }
}
