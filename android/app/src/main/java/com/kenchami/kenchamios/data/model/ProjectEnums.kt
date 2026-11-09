package com.kenchami.kenchamios.data.model

enum class Stage(override val wire: String) : WireEnum {
    Idea("idea"),
    Plan("plan"),
    Build("build"),
    Review("review"),
    Done("done"),
}

enum class DocKind(override val wire: String) : WireEnum {
    Idea("idea"),
    Plan("plan"),
    Adr("adr"),
    Note("note"),
}

enum class SavePointSource(override val wire: String) : WireEnum {
    Manual("manual"),
    SessionNote("session_note"),
}
