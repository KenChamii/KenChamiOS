package com.kenchami.kenchamios.context.platform

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import com.google.android.gms.common.api.ApiException
import com.google.android.gms.location.Geofence
import com.google.android.gms.location.GeofenceStatusCodes
import com.google.android.gms.location.GeofencingRequest
import com.google.android.gms.location.LocationServices
import com.kenchami.kenchamios.context.Geofences
import com.kenchami.kenchamios.context.PlaceFence
import com.kenchami.kenchamios.context.PlaceTransition

private const val DWELL_MS = 2 * 60 * 1000

/** Geofences through Play services: no location updates of our own, the system batches the checks. */
class PlayGeofences(private val context: Context) : Geofences {
    private val client = LocationServices.getGeofencingClient(context)

    override suspend fun replaceAll(places: List<PlaceFence>): String? {
        clear()
        if (places.isEmpty()) return null
        val request = GeofencingRequest.Builder().setInitialTrigger(0).addGeofences(places.map(::geofence)).build()
        return try {
            client.addGeofences(request, pendingIntent()).await()
            null
        } catch (error: ApiException) {
            describe(error.statusCode)
        } catch (_: SecurityException) {
            describe(GeofenceStatusCodes.GEOFENCE_INSUFFICIENT_LOCATION_PERMISSION)
        }
    }

    override suspend fun clear() {
        try {
            client.removeGeofences(pendingIntent()).await()
        } catch (_: ApiException) {
            // nothing registered, or Play services is missing: there is nothing to remove either way
        }
    }

    private fun geofence(fence: PlaceFence): Geofence {
        val place = fence.place
        val builder = Geofence.Builder()
            .setRequestId(fence.ruleId)
            .setCircularRegion(place.latitude, place.longitude, place.radiusM.toFloat())
            .setExpirationDuration(Geofence.NEVER_EXPIRE)
        return when (place.transition) {
            PlaceTransition.Arrive -> builder.setTransitionTypes(Geofence.GEOFENCE_TRANSITION_DWELL).setLoiteringDelay(DWELL_MS)
            PlaceTransition.Leave -> builder.setTransitionTypes(Geofence.GEOFENCE_TRANSITION_EXIT)
        }.build()
    }

    // the system fills in the transition, so the intent has to be mutable
    private fun pendingIntent(): PendingIntent = PendingIntent.getBroadcast(
        context,
        0,
        Intent(context, GeofenceReceiver::class.java),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE,
    )

    private fun describe(code: Int): String = when (code) {
        GeofenceStatusCodes.GEOFENCE_NOT_AVAILABLE -> "Location is turned off on this phone."
        GeofenceStatusCodes.GEOFENCE_TOO_MANY_GEOFENCES -> "Android cannot hold more places. Turn some place rules off."
        GeofenceStatusCodes.GEOFENCE_INSUFFICIENT_LOCATION_PERMISSION -> "Location permission is missing."
        else -> "Places could not be registered (code $code)."
    }
}

fun placeTransitionOf(geofenceTransition: Int): PlaceTransition? = when (geofenceTransition) {
    Geofence.GEOFENCE_TRANSITION_DWELL -> PlaceTransition.Arrive
    Geofence.GEOFENCE_TRANSITION_EXIT -> PlaceTransition.Leave
    else -> null
}
