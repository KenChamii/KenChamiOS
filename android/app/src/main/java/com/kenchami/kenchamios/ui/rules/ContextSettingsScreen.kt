package com.kenchami.kenchamios.ui.rules

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.SectionTitle

/** Which permission steps this Android version has. */
data class PermissionSteps(val background: Boolean, val notifications: Boolean) {
    companion object {
        fun current() = PermissionSteps(
            background = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q,
            notifications = Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU,
        )
    }
}

@Composable
fun ContextSettingsScreen(onBack: () -> Unit, viewModel: ContextSettingsViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val context = LocalContext.current
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) { viewModel.reload() }
    val foreground = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { viewModel.reload() }
    val single = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { viewModel.reload() }
    AppScreen("Rule settings", onBack = onBack) { padding ->
        ContextSettingsContent(
            state = state,
            steps = PermissionSteps.current(),
            onEnabled = viewModel::setEnabled,
            onAllowLocation = {
                foreground.launch(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION))
            },
            onAllowBackground = { single.launch(Manifest.permission.ACCESS_BACKGROUND_LOCATION) },
            onAllowNotifications = { single.launch(Manifest.permission.POST_NOTIFICATIONS) },
            onOpenSystemSettings = {
                context.startActivity(
                    Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", context.packageName, null)),
                )
            },
            padding = padding,
        )
    }
}

@Composable
fun ContextSettingsContent(
    state: ContextSettingsState,
    steps: PermissionSteps,
    onEnabled: (Boolean) -> Unit,
    onAllowLocation: () -> Unit,
    onAllowBackground: () -> Unit,
    onAllowNotifications: () -> Unit,
    onOpenSystemSettings: () -> Unit,
    padding: PaddingValues,
) {
    Column(
        Modifier.padding(padding).verticalScroll(rememberScrollState()).testTag("context-settings"),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(
            Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Switch(checked = state.enabled, onCheckedChange = onEnabled)
            Text("Context rules are on")
        }
        Info("Turn this off to stop every rule at once. Nothing runs and no place is registered while it is off.")

        SectionTitle("Location (place rules only)")
        Info(
            "A place rule needs the phone to tell the app when you enter or leave a circle. " +
                "Android does this with low-power geofences: the app never tracks you and never stores where you are. " +
                "Coordinates you type stay on this phone.",
        )
        Permission("Location", state.location, "Allow location", onAllowLocation)
        if (steps.background) {
            Info(
                "Rules must work while the app is closed, so Android asks for a second step. " +
                    "Choose Allow all the time on the next screen.",
            )
            Permission("Location all the time", state.backgroundLocation, "Allow all the time", onAllowBackground, enabled = state.location)
        }
        state.placeProblem?.let { Text(it, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(horizontal = 16.dp)) }

        SectionTitle("Notifications")
        Info("Reminders and checklists show a notification. Without it a checklist still appears on Today.")
        if (steps.notifications) {
            Permission("Notifications", state.notifications, "Allow notifications", onAllowNotifications)
        } else {
            Info(if (state.notifications) "Notifications: allowed" else "Notifications: turned off in system settings")
        }

        SectionTitle("Battery")
        Info(
            "Time rules use one inexact wake-up, so they can run a few minutes late. " +
                "Wi-Fi rules react to the system's own network events. Neither keeps the phone awake.",
        )
        FilledTonalButton(onClick = onOpenSystemSettings, modifier = Modifier.padding(horizontal = 16.dp)) {
            Text("Open system settings")
        }
    }
}

@Composable
private fun Info(text: String) {
    Text(text, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.padding(horizontal = 16.dp))
}

@Composable
private fun Permission(label: String, granted: Boolean, buttonLabel: String, onRequest: () -> Unit, enabled: Boolean = true) {
    Row(
        Modifier.padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("$label: ${if (granted) "allowed" else "not allowed"}", modifier = Modifier.weight(1f))
        if (!granted) Button(onClick = onRequest, enabled = enabled) { Text(buttonLabel) }
    }
}
