package com.kenchami.kenchamios.timer

import android.annotation.SuppressLint
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import com.kenchami.kenchamios.data.model.TimerStatus
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.time.Clock
import javax.inject.Inject

/** Keeps the focus timer alive with an ongoing notification while it runs or is paused. */
@AndroidEntryPoint
class TimerService : Service() {
    @Inject lateinit var timer: TimerController

    @Inject lateinit var notifications: TimerNotifications

    @Inject lateinit var clock: Clock

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private var following: Job? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // the service must show its notification right after it was started, before anything is loaded
        ServiceCompat.startForeground(
            this,
            NOTIFICATION_ID,
            notifications.ongoing(timer.state.value, clock.millis()),
            foregroundType(),
        )
        scope.launch {
            timer.recover()
            when (intent?.action) {
                ACTION_PAUSE -> timer.pause()
                ACTION_RESUME -> timer.resume()
                ACTION_STOP -> timer.stop()
            }
            if (following == null) following = launch { follow() }
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    private suspend fun follow() {
        timer.state.collectLatest { state ->
            if (state == null) {
                ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
                stopSelf()
                return@collectLatest
            }
            notifications.update(state, clock.millis())
            if (state.status == TimerStatus.Running) waitForPhaseEnd()
        }
    }

    // returns after a phase ended; a new state then restarts the collector
    private suspend fun waitForPhaseEnd() {
        while (true) {
            val running = timer.state.value ?: return
            delay(running.remainingMs(clock.millis()))
            val change = timer.tick()
            if (change != null) {
                notifications.phaseChanged(change)
                return
            }
        }
    }

    // ServiceCompat ignores the type before Android 10, so the constant is safe on API 26
    @SuppressLint("InlinedApi")
    private fun foregroundType(): Int = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
        ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
    } else {
        ServiceInfo.FOREGROUND_SERVICE_TYPE_MANIFEST
    }

    companion object {
        const val NOTIFICATION_ID = 1
        const val ACTION_PAUSE = "com.kenchami.kenchamios.timer.PAUSE"
        const val ACTION_RESUME = "com.kenchami.kenchamios.timer.RESUME"
        const val ACTION_STOP = "com.kenchami.kenchamios.timer.STOP"

        /** Call from the foreground app, after the timer was started or found running. */
        fun start(context: Context) {
            ContextCompat.startForegroundService(context, Intent(context, TimerService::class.java))
        }
    }
}
