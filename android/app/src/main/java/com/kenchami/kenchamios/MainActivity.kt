package com.kenchami.kenchamios

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.lifecycleScope
import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.sync.SyncTrigger
import com.kenchami.kenchamios.timer.TimerController
import com.kenchami.kenchamios.timer.TimerService
import com.kenchami.kenchamios.ui.common.UndoBus
import com.kenchami.kenchamios.ui.nav.AppScaffold
import com.kenchami.kenchamios.ui.theme.KenChamiTheme
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.launch
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    @Inject lateinit var timer: TimerController

    @Inject lateinit var syncTrigger: SyncTrigger

    @Inject lateinit var contextRules: ContextCoordinator

    @Inject lateinit var undo: UndoBus

    private val shared = mutableStateOf<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // after a rotation the share was already handed to quick add
        if (savedInstanceState == null) shared.value = sharedText(intent)
        syncTrigger.start()
        lifecycleScope.launch { contextRules.refresh() }
        lifecycleScope.launch {
            timer.recover()
            if (timer.state.value != null) TimerService.start(this@MainActivity)
        }
        enableEdgeToEdge()
        setContent {
            KenChamiTheme {
                AppScaffold(undo, shared.value, onSharedTextUsed = { shared.value = null })
            }
        }
    }
}
