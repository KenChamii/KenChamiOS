package com.kenchami.kenchamios.data.model

enum class TaskStatus(override val wire: String) : WireEnum {
    Todo("todo"),
    Doing("doing"),
    Review("review"),
    Done("done"),
}

enum class Priority(override val wire: String) : WireEnum {
    Low("low"),
    Medium("medium"),
    High("high"),
}
