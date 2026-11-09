package com.kenchami.kenchamios.context.platform

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.kenchami.kenchamios.context.ContextPermissions

class AndroidPermissions(private val context: Context) : ContextPermissions {
    override fun location(): Boolean = granted(Manifest.permission.ACCESS_FINE_LOCATION)

    override fun backgroundLocation(): Boolean =
        Build.VERSION.SDK_INT < Build.VERSION_CODES.Q || granted(Manifest.permission.ACCESS_BACKGROUND_LOCATION)

    override fun notifications(): Boolean = NotificationManagerCompat.from(context).areNotificationsEnabled()

    private fun granted(permission: String) = ContextCompat.checkSelfPermission(context, permission) == PackageManager.PERMISSION_GRANTED
}
