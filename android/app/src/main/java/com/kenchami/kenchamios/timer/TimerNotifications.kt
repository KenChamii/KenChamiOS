package com.kenchami.kenchamios.timer

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import com.kenchami.kenchamios.MainActivity
import com.kenchami.kenchamios.R
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.ui.common.formatClock
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

private const val ONGOING_CHANNEL = "focus_timer"
private const val ALERT_CHANNEL = "focus_phase"
private const val ALERT_ID = 2

@Singleton
class TimerNotifications @Inject constructor(@param:ApplicationContext private val context: Context) {
    private val manager = NotificationManagerCompat.from(context)

    init {
        manager.createNotificationChannel(
            NotificationChannelCompat.Builder(ONGOING_CHANNEL, NotificationManagerCompat.IMPORTANCE_LOW)
                .setName("Focus timer")
                .build(),
        )
        manager.createNotificationChannel(
            NotificationChannelCompat.Builder(ALERT_CHANNEL, NotificationManagerCompat.IMPORTANCE_DEFAULT)
                .setName("Focus phase changes")
                .build(),
        )
    }

    /** The ongoing notification of the service; a plain one while the timer is not loaded yet. */
    fun ongoing(state: TimerState?, now: Long): Notification {
        val builder = NotificationCompat.Builder(context, ONGOING_CHANNEL)
            .setSmallIcon(R.drawable.ic_timer)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setContentIntent(openApp())
        if (state == null) return builder.setContentTitle("Focus timer").build()

        builder.setContentTitle(phaseLabel(state.phase)).setContentText("Round ${state.round + 1}")
        if (state.status == TimerStatus.Running) {
            builder.setUsesChronometer(true)
                .setChronometerCountDown(true)
                .setShowWhen(true)
                .setWhen(now + state.remainingMs(now))
                .addAction(0, "Pause", action(TimerService.ACTION_PAUSE))
        } else {
            builder.setContentText("Paused, ${formatClock(state.remainingS(now))} left")
                .addAction(0, "Resume", action(TimerService.ACTION_RESUME))
        }
        return builder.addAction(0, "Stop", action(TimerService.ACTION_STOP)).build()
    }

    fun update(state: TimerState?, now: Long) {
        if (manager.areNotificationsEnabled()) manager.notify(TimerService.NOTIFICATION_ID, ongoing(state, now))
    }

    fun phaseChanged(change: PhaseChange) {
        if (!manager.areNotificationsEnabled()) return
        val text = when (change.next) {
            TimerPhase.Focus -> "Break over, time to focus"
            TimerPhase.Break -> "Focus finished, take a short break"
            TimerPhase.LongBreak -> "Focus finished, take a long break"
        }
        val notification = NotificationCompat.Builder(context, ALERT_CHANNEL)
            .setSmallIcon(R.drawable.ic_timer)
            .setContentTitle(phaseLabel(change.next))
            .setContentText(text)
            .setAutoCancel(true)
            .setContentIntent(openApp())
            .build()
        manager.notify(ALERT_ID, notification)
    }

    private fun phaseLabel(phase: TimerPhase): String = when (phase) {
        TimerPhase.Focus -> "Focus"
        TimerPhase.Break -> "Break"
        TimerPhase.LongBreak -> "Long break"
    }

    private fun openApp(): PendingIntent = PendingIntent.getActivity(
        context,
        0,
        Intent(context, MainActivity::class.java),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )

    private fun action(name: String): PendingIntent = PendingIntent.getForegroundService(
        context,
        name.hashCode(),
        Intent(context, TimerService::class.java).setAction(name),
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
}
