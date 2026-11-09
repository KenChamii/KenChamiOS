package com.kenchami.kenchamios.data.model

enum class TimeEntryKind(override val wire: String) : WireEnum {
    Focus("focus"),
    Break("break"),
    Manual("manual"),
}

enum class TimerPhase(override val wire: String) : WireEnum {
    Focus("focus"),
    Break("break"),
    LongBreak("long_break"),
}

enum class TimerStatus(override val wire: String) : WireEnum {
    Running("running"),
    Paused("paused"),
}
