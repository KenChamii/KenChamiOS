package com.kenchami.kenchamios.data.db

import androidx.room.TypeConverter
import com.kenchami.kenchamios.core.decodeStringList
import com.kenchami.kenchamios.core.encodeStringList
import com.kenchami.kenchamios.data.model.DocKind
import com.kenchami.kenchamios.data.model.MediaSource
import com.kenchami.kenchamios.data.model.MediaStatus
import com.kenchami.kenchamios.data.model.MediaType
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.model.SavePointSource
import com.kenchami.kenchamios.data.model.Stage
import com.kenchami.kenchamios.data.model.TaskStatus
import com.kenchami.kenchamios.data.model.TimeEntryKind
import com.kenchami.kenchamios.data.model.TimerPhase
import com.kenchami.kenchamios.data.model.TimerStatus
import com.kenchami.kenchamios.data.model.fromWire

class Converters {
    @TypeConverter fun stringList(value: List<String>): String = encodeStringList(value)

    @TypeConverter fun stringList(value: String): List<String> = decodeStringList(value)

    @TypeConverter fun stage(value: Stage): String = value.wire

    @TypeConverter fun stage(value: String): Stage = fromWire(value)

    @TypeConverter fun docKind(value: DocKind): String = value.wire

    @TypeConverter fun docKind(value: String): DocKind = fromWire(value)

    @TypeConverter fun savePointSource(value: SavePointSource): String = value.wire

    @TypeConverter fun savePointSource(value: String): SavePointSource = fromWire(value)

    @TypeConverter fun taskStatus(value: TaskStatus): String = value.wire

    @TypeConverter fun taskStatus(value: String): TaskStatus = fromWire(value)

    @TypeConverter fun priority(value: Priority): String = value.wire

    @TypeConverter fun priority(value: String): Priority = fromWire(value)

    @TypeConverter fun mediaType(value: MediaType): String = value.wire

    @TypeConverter fun mediaType(value: String): MediaType = fromWire(value)

    @TypeConverter fun mediaStatus(value: MediaStatus): String = value.wire

    @TypeConverter fun mediaStatus(value: String): MediaStatus = fromWire(value)

    @TypeConverter fun mediaSource(value: MediaSource): String = value.wire

    @TypeConverter fun mediaSource(value: String): MediaSource = fromWire(value)

    @TypeConverter fun timeEntryKind(value: TimeEntryKind): String = value.wire

    @TypeConverter fun timeEntryKind(value: String): TimeEntryKind = fromWire(value)

    @TypeConverter fun timerPhase(value: TimerPhase): String = value.wire

    @TypeConverter fun timerPhase(value: String): TimerPhase = fromWire(value)

    @TypeConverter fun timerStatus(value: TimerStatus): String = value.wire

    @TypeConverter fun timerStatus(value: String): TimerStatus = fromWire(value)
}
