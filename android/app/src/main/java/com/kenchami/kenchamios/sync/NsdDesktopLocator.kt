package com.kenchami.kenchamios.sync

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.coroutines.resume

private const val SERVICE_TYPE = "_kenchamios._tcp"
private const val SEARCH_TIMEOUT_MS = 8_000L

/** Looks for the desktop with the platform network service discovery (mDNS). */
class NsdDesktopLocator(context: Context) : DesktopLocator {
    private val manager = context.getSystemService(Context.NSD_SERVICE) as NsdManager

    override suspend fun find(): Endpoint? = withTimeoutOrNull(SEARCH_TIMEOUT_MS) {
        suspendCancellableCoroutine { continuation ->
            val listener = object : NsdManager.DiscoveryListener {
                override fun onServiceFound(service: NsdServiceInfo) {
                    resolve(service) { endpoint -> if (continuation.isActive) continuation.resume(endpoint) }
                }

                override fun onStartDiscoveryFailed(serviceType: String?, errorCode: Int) {
                    if (continuation.isActive) continuation.resume(null)
                }

                override fun onStopDiscoveryFailed(serviceType: String?, errorCode: Int) = Unit

                override fun onDiscoveryStarted(serviceType: String?) = Unit

                override fun onDiscoveryStopped(serviceType: String?) = Unit

                override fun onServiceLost(service: NsdServiceInfo?) = Unit
            }
            manager.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, listener)
            continuation.invokeOnCancellation { stop(listener) }
        }
    }

    private fun stop(listener: NsdManager.DiscoveryListener) {
        try {
            manager.stopServiceDiscovery(listener)
        } catch (_: IllegalArgumentException) {
            // discovery had already ended
        }
    }

    @Suppress("DEPRECATION")
    private fun resolve(service: NsdServiceInfo, done: (Endpoint?) -> Unit) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val callback = object : NsdManager.ServiceInfoCallback {
                override fun onServiceUpdated(info: NsdServiceInfo) {
                    done(info.hostAddresses.firstOrNull()?.hostAddress?.let { Endpoint(it, info.port) })
                    manager.unregisterServiceInfoCallback(this)
                }

                override fun onServiceInfoCallbackRegistrationFailed(errorCode: Int) = done(null)

                override fun onServiceLost() = Unit

                override fun onServiceInfoCallbackUnregistered() = Unit
            }
            manager.registerServiceInfoCallback(service, Runnable::run, callback)
        } else {
            manager.resolveService(
                service,
                object : NsdManager.ResolveListener {
                    override fun onResolveFailed(serviceInfo: NsdServiceInfo?, errorCode: Int) = done(null)

                    override fun onServiceResolved(info: NsdServiceInfo) {
                        done(info.host?.hostAddress?.let { Endpoint(it, info.port) })
                    }
                },
            )
        }
    }
}
