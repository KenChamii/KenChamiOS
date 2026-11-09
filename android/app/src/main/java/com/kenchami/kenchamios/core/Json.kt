package com.kenchami.kenchamios.core

import org.json.JSONArray
import org.json.JSONTokener

// the columns that hold json (tags, settings) use the same text form as the desktop

fun encodeStringList(values: List<String>): String = JSONArray(values).toString()

fun decodeStringList(json: String): List<String> {
    val array = JSONArray(json)
    return List(array.length()) { array.getString(it) }
}

fun encodeString(value: String): String = JSONArray().put(value).toString().removeSurrounding("[", "]")

fun decodeString(json: String): String = JSONTokener(json).nextValue() as String
