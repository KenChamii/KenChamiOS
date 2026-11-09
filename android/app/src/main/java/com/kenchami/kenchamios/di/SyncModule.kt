package com.kenchami.kenchamios.di

import android.content.Context
import android.os.Build
import androidx.work.WorkManager
import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.data.Stamper
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.dao.ChangeLogDao
import com.kenchami.kenchamios.sync.AppScope
import com.kenchami.kenchamios.sync.DesktopLocator
import com.kenchami.kenchamios.sync.KeystoreSecretStore
import com.kenchami.kenchamios.sync.NsdDesktopLocator
import com.kenchami.kenchamios.sync.PinnedApiFactory
import com.kenchami.kenchamios.sync.SecretStore
import com.kenchami.kenchamios.sync.SyncApiFactory
import com.kenchami.kenchamios.sync.SyncScheduler
import com.kenchami.kenchamios.sync.SyncSchema
import com.kenchami.kenchamios.sync.SyncStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import java.time.Clock
import javax.inject.Named
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object SyncModule {
    @Provides
    @Singleton
    @AppScope
    fun appScope(): CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    @Provides
    @Singleton
    fun schema(@ApplicationContext context: Context): SyncSchema {
        fun asset(name: String) = context.assets.open(name).bufferedReader().use { it.readText() }
        return SyncSchema.parse(asset("tables.schema.json"), asset("enums.schema.json"))
    }

    @Provides
    @Singleton
    fun store(database: AppDatabase, schema: SyncSchema, hlc: Hlc, clock: Clock, stamper: Stamper) =
        SyncStore(database, schema, hlc, clock, stamper)

    @Provides
    @Singleton
    fun secrets(@ApplicationContext context: Context): SecretStore = KeystoreSecretStore(context)

    @Provides
    fun apiFactory(): SyncApiFactory = PinnedApiFactory

    @Provides
    @Singleton
    fun locator(@ApplicationContext context: Context): DesktopLocator = NsdDesktopLocator(context)

    @Provides
    @Singleton
    fun scheduler(@ApplicationContext context: Context) = SyncScheduler(WorkManager.getInstance(context))

    @Provides
    fun changeLogDao(database: AppDatabase): ChangeLogDao = database.changeLogDao()

    @Provides
    @Named("deviceName")
    fun deviceName(): String = Build.MODEL.take(MAX_NAME_LENGTH).ifBlank { "Android phone" }

    private const val MAX_NAME_LENGTH = 64
}
