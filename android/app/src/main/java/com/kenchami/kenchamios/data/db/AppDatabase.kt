package com.kenchami.kenchamios.data.db

import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.TypeConverters
import com.kenchami.kenchamios.data.db.dao.ActivityDao
import com.kenchami.kenchamios.data.db.dao.ChangeLogDao
import com.kenchami.kenchamios.data.db.dao.MediaDao
import com.kenchami.kenchamios.data.db.dao.ProjectChildDao
import com.kenchami.kenchamios.data.db.dao.ProjectDao
import com.kenchami.kenchamios.data.db.dao.PromptDao
import com.kenchami.kenchamios.data.db.dao.SavePointDao
import com.kenchami.kenchamios.data.db.dao.SettingDao
import com.kenchami.kenchamios.data.db.dao.TaskDao
import com.kenchami.kenchamios.data.db.dao.TimerDao
import com.kenchami.kenchamios.data.db.entity.ActivityEntity
import com.kenchami.kenchamios.data.db.entity.ChangeLogEntity
import com.kenchami.kenchamios.data.db.entity.ChecklistItemEntity
import com.kenchami.kenchamios.data.db.entity.MediaEntryEntity
import com.kenchami.kenchamios.data.db.entity.MediaItemEntity
import com.kenchami.kenchamios.data.db.entity.MediaProgressLogEntity
import com.kenchami.kenchamios.data.db.entity.ProjectDocEntity
import com.kenchami.kenchamios.data.db.entity.ProjectEntity
import com.kenchami.kenchamios.data.db.entity.PromptEntity
import com.kenchami.kenchamios.data.db.entity.SavePointEntity
import com.kenchami.kenchamios.data.db.entity.SettingEntity
import com.kenchami.kenchamios.data.db.entity.TaskEntity
import com.kenchami.kenchamios.data.db.entity.TimeEntryEntity
import com.kenchami.kenchamios.data.db.entity.TimerStateEntity

@Database(
    entities = [
        SettingEntity::class,
        ActivityEntity::class,
        ProjectEntity::class,
        ProjectDocEntity::class,
        TaskEntity::class,
        ChecklistItemEntity::class,
        SavePointEntity::class,
        PromptEntity::class,
        TimeEntryEntity::class,
        TimerStateEntity::class,
        MediaItemEntity::class,
        MediaEntryEntity::class,
        MediaProgressLogEntity::class,
        ChangeLogEntity::class,
    ],
    version = 2,
)
@TypeConverters(Converters::class)
abstract class AppDatabase : RoomDatabase() {
    abstract fun settingDao(): SettingDao
    abstract fun activityDao(): ActivityDao
    abstract fun projectDao(): ProjectDao
    abstract fun projectChildDao(): ProjectChildDao
    abstract fun taskDao(): TaskDao
    abstract fun savePointDao(): SavePointDao
    abstract fun promptDao(): PromptDao
    abstract fun mediaDao(): MediaDao
    abstract fun timerDao(): TimerDao
    abstract fun changeLogDao(): ChangeLogDao

    companion object {
        const val NAME = "kenchamios.db"
    }
}
