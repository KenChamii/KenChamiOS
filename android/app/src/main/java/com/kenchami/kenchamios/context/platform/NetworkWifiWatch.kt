package com.kenchami.kenchamios.context.platform

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import com.kenchami.kenchamios.context.WifiWatch

/** A system callback that wakes the receiver when a Wi-Fi network appears or goes; no SSID, so no location permission. */
class NetworkWifiWatch(private val context: Context) : WifiWatch {
    private val manager = context.getSystemService(ConnectivityManager::class.java)

    override fun start() {
        val request = NetworkRequest.Builder().addTransportType(NetworkCapabilities.TRANSPORT_WIFI).build()
        manager.registerNetworkCallback(request, pendingIntent())
    }

    override fun stop() {
        try {
            manager.unregisterNetworkCallback(pendingIntent())
        } catch (_: IllegalArgumentException) {
            // it was not registered
        }
    }

    override fun connected(): Boolean =
        manager.getNetworkCapabilities(manager.activeNetwork)?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true

    // the system adds the network to the intent, so it has to be mutable
    private fun pendingIntent(): PendingIntent = PendingIntent.getBroadcast(
        context,
        0,
        Intent(context, WifiReceiver::class.java),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE,
    )
}
