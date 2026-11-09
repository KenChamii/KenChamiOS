package com.kenchami.kenchamios.ui.projects

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.core.content.ContextCompat

/**
 * Returns a function that starts a focus timer for a task. On Android 13 and later it asks for the
 * notification permission first; the timer starts either way, a refusal only hides its notification.
 */
@Composable
fun rememberFocusStarter(onStart: (taskId: String) -> Unit): (String) -> Unit {
    val context = LocalContext.current
    var pending by remember { mutableStateOf<String?>(null) }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {
        pending?.let(onStart)
        pending = null
    }
    return { taskId ->
        val granted = Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED
        if (granted) {
            onStart(taskId)
        } else {
            pending = taskId
            launcher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }
}
