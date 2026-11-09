package com.kenchami.kenchamios.data.model

enum class MediaType(override val wire: String) : WireEnum {
    Anime("anime"),
    Manga("manga"),
    Game("game"),
    Movie("movie"),
    Show("show"),
    Album("album"),
    Artist("artist"),
}

enum class MediaStatus(override val wire: String) : WireEnum {
    InProgress("in_progress"),
    Planned("planned"),
    Completed("completed"),
    OnHold("on_hold"),
    Dropped("dropped"),
}

enum class MediaSource(override val wire: String) : WireEnum {
    Anilist("anilist"),
    Rawg("rawg"),
    Tmdb("tmdb"),
    Musicbrainz("musicbrainz"),
    Manual("manual"),
}
