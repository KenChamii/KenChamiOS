package com.kenchami.kenchamios.context.platform

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import com.google.android.gms.location.GeofencingEvent
import com.kenchami.kenchamios.context.ContextCoordinator
import com.kenchami.kenchamios.context.ContextEvent
import com.kenchami.kenchamios.context.ContextRunner
import com.kenchami.kenchamios.context.WifiWatch
import com.kenchami.kenchamios.sync.AppScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import javax.inject.Inject

/** Keeps the process alive for [block], which the system would otherwise end when onReceive returns. */
private fun BroadcastReceiver.async(scope: CoroutineScope, block: suspend () -> Unit) {
    val pending = goAsync()
    scope.launch {
        try {
            block()
        } finally {
            pending.finish()
        }
    }
}

@AndroidEntryPoint
class GeofenceReceiver : BroadcastReceiver() {
    @Inject lateinit var runner: ContextRunner

    @Inject @AppScope
    lateinit var scope: CoroutineScope

    override fun onReceive(context: Context, intent: Intent) {
        val event = GeofencingEvent.fromIntent(intent)?.takeUnless { it.hasError() } ?: return
        val transition = placeTransitionOf(event.geofenceTransition) ?: return
        val ids = event.triggeringGeofences.orEmpty().map { it.requestId }
        async(scope) { ids.forEach { runner.handle(ContextEvent.PlaceChanged(it, transition)) } }
    }
}

@AndroidEntryPoint
class WifiReceiver : BroadcastReceiver() {
    @Inject lateinit var runner: ContextRunner

    @Inject lateinit var wifi: WifiWatch

    @Inject @AppScope
    lateinit var scope: CoroutineScope

    override fun onReceive(context: Context, intent: Intent) = async(scope) { runner.onWifiState(wifi.connected()) }
}

/** After a reboot, an update or a clock change the geofences, the wake-up and the Wi-Fi watch have to be set up again. */
@AndroidEntryPoint
class ContextResetReceiver : BroadcastReceiver() {
    @Inject lateinit var coordinator: ContextCoordinator

    @Inject @AppScope
    lateinit var scope: CoroutineScope

    override fun onReceive(context: Context, intent: Intent) = async(scope) { coordinator.refresh() }
}
