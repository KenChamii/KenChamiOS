package com.kenchami.kenchamios.context.platform

import android.annotation.SuppressLint
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import com.kenchami.kenchamios.MainActivity
import com.kenchami.kenchamios.R
import com.kenchami.kenchamios.context.RuleNotifier

private const val CHANNEL = "context_rules"

/** One notification per rule; a repeated trigger replaces the old one instead of stacking. */
class ContextNotifier(private val context: Context) : RuleNotifier {
    private val manager = NotificationManagerCompat.from(context)

    init {
        manager.createNotificationChannel(
            NotificationChannelCompat.Builder(CHANNEL, NotificationManagerCompat.IMPORTANCE_DEFAULT).setName("Context rules").build(),
        )
    }

    // areNotificationsEnabled covers the Android 13 permission
    @SuppressLint("MissingPermission")
    override fun post(ruleId: String, title: String, text: String) {
        if (!manager.areNotificationsEnabled()) return
        val notification = NotificationCompat.Builder(context, CHANNEL)
            .setSmallIcon(R.drawable.ic_context)
            .setContentTitle(title)
            .setContentText(text)
            .setStyle(NotificationCompat.BigTextStyle().bigText(text))
            .setAutoCancel(true)
            .setContentIntent(openApp())
            .build()
        manager.notify(ruleId.hashCode(), notification)
    }

    private fun openApp(): PendingIntent = PendingIntent.getActivity(
        context,
        0,
        Intent(context, MainActivity::class.java),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
}
