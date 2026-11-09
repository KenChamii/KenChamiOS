package com.kenchami.kenchamios.di

import android.content.Context
import androidx.room.Room
import com.kenchami.kenchamios.core.Hlc
import com.kenchami.kenchamios.core.Uuid7Generator
import com.kenchami.kenchamios.data.db.AppDatabase
import com.kenchami.kenchamios.data.db.ChangeLogCallback
import com.kenchami.kenchamios.data.db.MIGRATION_1_2
import com.kenchami.kenchamios.data.db.dao.ActivityDao
import com.kenchami.kenchamios.data.db.dao.SettingDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import java.time.Clock
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DataModule {
    @Provides
    @Singleton
    fun clock(): Clock = Clock.systemDefaultZone()

    @Provides
    @Singleton
    fun hlc(clock: Clock): Hlc = Hlc(clock)

    @Provides
    @Singleton
    fun uuids(clock: Clock): Uuid7Generator = Uuid7Generator(clock)

    @Provides
    @Singleton
    fun database(@ApplicationContext context: Context, hlc: Hlc): AppDatabase =
        Room.databaseBuilder(context, AppDatabase::class.java, AppDatabase.NAME)
            .addMigrations(MIGRATION_1_2)
            .addCallback(ChangeLogCallback(hlc))
            .build()

    @Provides
    fun settingDao(database: AppDatabase): SettingDao = database.settingDao()

    @Provides
    fun activityDao(database: AppDatabase): ActivityDao = database.activityDao()
}
