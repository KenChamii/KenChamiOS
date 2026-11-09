package com.kenchami.kenchamios.ui.sync

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.kenchami.kenchamios.sync.SyncSnapshot
import com.kenchami.kenchamios.ui.common.AppScreen
import com.kenchami.kenchamios.ui.common.AppTextField
import com.kenchami.kenchamios.ui.common.ConfirmDialog
import com.kenchami.kenchamios.ui.common.SectionTitle
import com.kenchami.kenchamios.ui.common.timeAgo

@Composable
fun SyncScreen(onBack: () -> Unit, viewModel: SyncViewModel = hiltViewModel()) {
    val snapshot by viewModel.snapshot.collectAsStateWithLifecycle()
    val progress by viewModel.progress.collectAsStateWithLifecycle()
    val context = LocalContext.current
    AppScreen("Sync", onBack = onBack) { padding ->
        SyncContent(
            snapshot = snapshot,
            progress = progress,
            now = System.currentTimeMillis(),
            onSyncNow = viewModel::syncNow,
            onPair = viewModel::pair,
            onScan = {
                scanQrCode(context, onResult = viewModel::pair, onError = {
                    viewModel.showMessage("The scanner is not available. Paste the pairing text instead.")
                })
            },
            onUnpair = viewModel::unpair,
            padding = padding,
        )
    }
}

@Composable
fun SyncContent(
    snapshot: SyncSnapshot,
    progress: SyncProgress,
    now: Long,
    onSyncNow: () -> Unit,
    onPair: (String) -> Unit,
    onScan: () -> Unit,
    onUnpair: () -> Unit,
    padding: PaddingValues,
) {
    var confirmUnpair by rememberSaveable { mutableStateOf(false) }
    Column(
        Modifier.padding(padding).verticalScroll(rememberScrollState()).testTag("sync"),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        val link = snapshot.link
        SectionTitle("Desktop")
        if (link == null) {
            Line("Not paired. Sync works over your home Wi-Fi with the KenChamiOS desktop app.")
        } else {
            Line("Paired with ${link.host}:${link.port}")
            Line(snapshot.lastSyncAt?.let { "Last sync ${timeAgo(it, now)}" } ?: "Never synced")
            Button(onClick = onSyncNow, enabled = !progress.busy, modifier = Modifier.padding(horizontal = 16.dp)) {
                Text(if (progress.busy) "Working..." else "Sync now")
            }
            FilledTonalButton(
                onClick = { confirmUnpair = true },
                enabled = !progress.busy,
                modifier = Modifier.padding(horizontal = 16.dp),
            ) {
                Text("Unpair")
            }
        }
        snapshot.lastError?.let { Line("Last problem: $it", MaterialTheme.colorScheme.error) }
        progress.message?.let { Line(it) }
        SectionTitle("Pair a desktop")
        PairForm(enabled = !progress.busy, onPair = onPair, onScan = onScan)
    }
    if (confirmUnpair) {
        ConfirmDialog(
            title = "Unpair?",
            text = "This phone stops syncing. Your data stays on both devices.",
            confirmLabel = "Unpair",
            onConfirm = {
                confirmUnpair = false
                onUnpair()
            },
            onDismiss = { confirmUnpair = false },
        )
    }
}

@Composable
private fun Line(text: String, color: Color = MaterialTheme.colorScheme.onSurface) {
    Text(text, style = MaterialTheme.typography.bodyMedium, color = color, modifier = Modifier.padding(horizontal = 16.dp, vertical = 2.dp))
}

@Composable
private fun PairForm(enabled: Boolean, onPair: (String) -> Unit, onScan: () -> Unit) {
    var text by rememberSaveable { mutableStateOf("") }
    Column(Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            "On the desktop open Settings, Phone sync, Pair a phone. Then scan its QR code.",
            style = MaterialTheme.typography.bodyMedium,
        )
        Button(onClick = onScan, enabled = enabled) { Text("Scan QR code") }
        AppTextField(
            value = text,
            onValueChange = { text = it },
            label = "Or paste the pairing text",
            modifier = Modifier.fillMaxWidth(),
        )
        FilledTonalButton(onClick = { onPair(text) }, enabled = enabled && text.isNotBlank()) { Text("Pair with pasted text") }
    }
}
