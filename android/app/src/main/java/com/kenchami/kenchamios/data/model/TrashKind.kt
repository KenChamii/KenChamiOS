package com.kenchami.kenchamios.data.model

/** What a trash row is; [wire] is the kind the trash query returns. */
enum class TrashKind(override val wire: String) : WireEnum {
    Project("project"),
    Task("task"),
    Prompt("prompt"),
    SavePoint("save_point"),
    Media("media"),
    Habit("habit"),
}
