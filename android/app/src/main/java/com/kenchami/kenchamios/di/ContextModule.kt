package com.kenchami.kenchamios.di

import android.content.Context
import androidx.work.WorkManager
import com.kenchami.kenchamios.context.ContextPermissions
import com.kenchami.kenchamios.context.DueTasks
import com.kenchami.kenchamios.context.Geofences
import com.kenchami.kenchamios.context.RuleNotifier
import com.kenchami.kenchamios.context.TimePlan
import com.kenchami.kenchamios.context.WifiWatch
import com.kenchami.kenchamios.context.platform.AndroidPermissions
import com.kenchami.kenchamios.context.platform.ContextNotifier
import com.kenchami.kenchamios.context.platform.NetworkWifiWatch
import com.kenchami.kenchamios.context.platform.PlayGeofences
import com.kenchami.kenchamios.context.platform.WorkTimePlan
import com.kenchami.kenchamios.core.dayBounds
import com.kenchami.kenchamios.data.repo.TaskRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.flow.first
import java.time.Clock
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object ContextModule {
    @Provides
    @Singleton
    fun geofences(@ApplicationContext context: Context): Geofences = PlayGeofences(context)

    @Provides
    @Singleton
    fun timePlan(@ApplicationContext context: Context, clock: Clock): TimePlan = WorkTimePlan(WorkManager.getInstance(context), clock)

    @Provides
    @Singleton
    fun wifiWatch(@ApplicationContext context: Context): WifiWatch = NetworkWifiWatch(context)

    @Provides
    @Singleton
    fun permissions(@ApplicationContext context: Context): ContextPermissions = AndroidPermissions(context)

    @Provides
    @Singleton
    fun notifier(@ApplicationContext context: Context): RuleNotifier = ContextNotifier(context)

    @Provides
    fun dueTasks(tasks: TaskRepository, clock: Clock): DueTasks = DueTasks {
        tasks.observeDue(dayBounds(clock.millis(), clock.zone).end).first().map { it.title }
    }
}
