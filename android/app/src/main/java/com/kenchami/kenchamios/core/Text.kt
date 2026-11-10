package com.kenchami.kenchamios.core

private val NON_ALPHANUMERIC = Regex("[^A-Za-z0-9]+")

/** Lowercase ascii words joined by dashes, e.g. "My App!" becomes "my-app". */
fun slugify(name: String): String = name.split(NON_ALPHANUMERIC)
    .filter { it.isNotEmpty() }
    .joinToString("-") { it.lowercase() }
    .ifEmpty { "project" }

/** A `LIKE ... ESCAPE '\'` pattern that matches [text] anywhere, with its wildcards taken literally. */
fun likePattern(text: String): String = "%" + text.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_") + "%"

/** Trims tags, drops empty ones and removes duplicates, keeping the first. */
fun cleanTags(tags: List<String>): List<String> = tags.map { it.trim() }.filter { it.isNotEmpty() }.distinct()
