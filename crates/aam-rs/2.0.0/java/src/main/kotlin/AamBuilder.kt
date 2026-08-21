package com.rustgames.aam

import java.io.File

/** Fluent builder for creating AAM content programmatically. */
class AamBuilder(capacity: Int = 0) {
    private val buffer = StringBuilder(if (capacity > 0) capacity else 16)

    data class SchemaField(val name: String, val typeName: String, val optional: Boolean = false) {
        fun toAam(): String = if (optional) "$name*: $typeName" else "$name: $typeName"

        companion object {
            @JvmStatic
            fun required(name: String, typeName: String): SchemaField = SchemaField(name, typeName, false)

            @JvmStatic
            fun optional(name: String, typeName: String): SchemaField = SchemaField(name, typeName, true)
        }
    }

    fun addLine(key: String, value: String): AamBuilder {
        pushSeparator()
        buffer.append(key).append(" = ").append(value)
        return this
    }

    fun comment(text: String): AamBuilder {
        pushSeparator()
        buffer.append("# ").append(text)
        return this
    }

    fun schema(name: String, fields: Iterable<SchemaField>): AamBuilder {
        pushSeparator()
        buffer.append("@schema ").append(name).append(" { ")
        fields.forEachIndexed { index, field ->
            if (index > 0) {
                buffer.append(", ")
            }
            buffer.append(field.toAam())
        }
        buffer.append(" }")
        return this
    }

    fun derive(path: String, schemas: Iterable<String> = emptyList()): AamBuilder {
        pushSeparator()
        buffer.append("@derive ").append(path)
        schemas.forEach { schema -> buffer.append("::").append(schema) }
        return this
    }

    fun import(path: String): AamBuilder {
        pushSeparator()
        buffer.append("@import ").append(path)
        return this
    }

    fun typeAlias(alias: String, typeName: String): AamBuilder {
        pushSeparator()
        buffer.append("@type ").append(alias).append(" = ").append(typeName)
        return this
    }

    fun build(): String = buffer.toString()

    fun toFile(path: String) {
        File(path).writeText(buffer.toString())
    }

    override fun toString(): String = buffer.toString()

    private fun pushSeparator() {
        if (buffer.isNotEmpty()) {
            buffer.append('\n')
        }
    }
}

