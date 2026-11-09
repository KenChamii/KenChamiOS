package com.kenchami.kenchamios.data.model

/** An enum stored as the snake case text that `shared/schema/enums.schema.json` lists. */
interface WireEnum {
    val wire: String
}

inline fun <reified E> fromWire(value: String): E where E : Enum<E>, E : WireEnum =
    enumValues<E>().firstOrNull { it.wire == value } ?: error("unknown ${E::class.simpleName} value: $value")
